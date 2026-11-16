//! Command admission, single-use previews and ordered action execution.
use super::App;
use crate::planner::{self, Plan};
#[cfg(feature = "voice")]
use crate::protocol::VoiceMode;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    sync::{atomic::Ordering, Arc},
    time::{Duration, Instant},
};

pub(super) struct Pending {
    token: u64,
    plan: Plan,
    created: Instant,
    active_window: Option<String>,
}

impl App {
    pub(super) async fn cancel(&self) -> Result<()> {
        if self.busy.available_permits() == 0 {
            bail!("a command is running; dismiss its preview after planning completes");
        }
        self.pending.lock().await.take();
        self.update(|state| {
            state.confirmation = None;
            state.actions.clear();
            state.phase = "ready".into();
            state.message = "Preview dismissed".into();
        });
        Ok(())
    }

    pub(super) async fn confirm(self: &Arc<Self>, token: u64) -> Result<Value> {
        let permit = self
            .busy
            .clone()
            .try_acquire_owned()
            .context("another command is running")?;
        let mut pending = self.pending.lock().await;
        let plan = pending
            .as_ref()
            .context("no action is waiting for confirmation")?;
        if plan.token != token {
            bail!("confirmation token does not match");
        }
        let plan = pending.take().unwrap();
        drop(pending);
        self.update(|state| state.confirmation = None);
        if plan.created.elapsed() > Duration::from_secs(30) {
            self.fail("Preview expired; request the action again");
            bail!("preview expired after 30 seconds");
        }
        if let Some(address) = &plan.active_window {
            let connection = self
                .desktop
                .connection
                .as_ref()
                .context("Hyprland disconnected")?;
            if !connection
                .active_window()
                .await?
                .is_some_and(|window| window.address == *address)
            {
                self.fail("Window focus changed; request the move again");
                bail!("window focus changed since preview");
            }
        }
        let app = self.clone();
        tokio::spawn(async move {
            if let Err(error) = app.execute(&plan.plan, plan.active_window.as_deref()).await {
                app.fail(&error.to_string());
            }
            drop(permit);
        });
        Ok(json!({"accepted":true}))
    }

    pub(super) async fn submit(self: &Arc<Self>, text: String, dry_run: bool) -> Result<Value> {
        let permit = self
            .busy
            .clone()
            .try_acquire_owned()
            .context("another command is running")?;
        if self.pending.lock().await.is_some() {
            bail!("confirm or dismiss the current preview first");
        }
        // Validate the text before publishing it or sending it to a model.
        let direct = planner::direct(&text, &self.config)?;
        self.update(|s| {
            s.phase = "planning".into();
            s.message = "Preparing your action".into();
            s.transcript = text.clone();
            s.receipts.clear();
            s.actions.clear();
            s.confirmation = None;
        });
        let app = self.clone();
        tokio::spawn(async move {
            if let Err(error) = app.prepare_command(text, direct, dry_run).await {
                app.fail(&error.to_string());
            }
            drop(permit);
        });
        Ok(json!({"accepted":true}))
    }

    async fn prepare_command(
        &self,
        text: String,
        direct: Option<Plan>,
        dry_run: bool,
    ) -> Result<()> {
        let plan = match direct {
            Some(plan) => plan,
            None => planner::plan(&text, &self.config).await?,
        };
        let active_window = if plan
            .actions
            .iter()
            .any(|action| matches!(action, planner::Action::MoveWindow { .. }))
        {
            if let Some(connection) = &self.desktop.connection {
                connection
                    .active_window()
                    .await?
                    .map(|window| window.address)
            } else {
                None
            }
        } else {
            None
        };
        self.update(|state| {
            state.actions = plan
                .actions
                .iter()
                .map(|action| action.description())
                .collect()
        });
        if dry_run || plan.source == "local_llm" {
            let token = self.sequence.fetch_add(1, Ordering::Relaxed);
            *self.pending.lock().await = Some(Pending {
                token,
                plan,
                created: Instant::now(),
                active_window,
            });
            self.update(|state| {
                state.phase = "review".into();
                state.message = "Review the action before running it".into();
                state.confirmation = Some(token);
                state.panel_open = true;
            });
        } else {
            self.execute(&plan, active_window.as_deref()).await?;
        }
        Ok(())
    }

    async fn execute(&self, plan: &Plan, target: Option<&str>) -> Result<()> {
        self.update(|s| {
            s.phase = "executing".into();
            s.message = "Applying your action".into();
        });
        for action in &plan.actions {
            let receipt = self.desktop.execute(action, &self.config, target).await?;
            self.update(|s| s.receipts.push(receipt));
        }
        self.update(|s| {
            s.phase = "done".into();
            s.message = if self.desktop.demo {
                "Demo complete · no desktop changes"
            } else {
                "Action completed"
            }
            .into();
        });
        #[cfg(feature = "voice")]
        if self.config.voice.speak_results && self.state().voice == VoiceMode::Wake {
            if let Err(e) = self.feedback().await {
                tracing::warn!(error = %e, "spoken feedback unavailable");
                self.update(|s| {
                    s.phase = "done".into();
                    s.message = "Action completed · spoken feedback unavailable".into();
                });
            }
        }
        Ok(())
    }
}
