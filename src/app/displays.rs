use super::App;
use crate::{
    protocol::DesktopPanel,
    services::displays::{self, Request},
};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::sync::{oneshot, OwnedMutexGuard};

impl App {
    pub(super) fn display(self: &Arc<Self>, request: Request) -> Result<Value> {
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        ensure!(
            !self.desktop.demo && self.desktop.connection.is_some(),
            "Display settings need the active Hyprland session"
        );
        match request {
            Request::Keep { token } => self.displays.decide(token, true)?,
            Request::Revert { token } => self.displays.decide(token, false)?,
            Request::Refresh => self.start_display(None, false)?,
            Request::Preview { revision, changes } => {
                let state = self.state();
                ensure!(
                    state.desktop_panel == Some(DesktopPanel::Settings),
                    "Open display settings before previewing a layout"
                );
                ensure!(
                    state.displays.revision == revision,
                    "Outputs changed. Review the layout again."
                );
                self.start_display(Some(changes), false)?;
            }
        }
        Ok(json!({"accepted":true}))
    }

    pub(super) fn outputs_changed(&self) {
        if self.displays.managed || self.state().desktop_panel == Some(DesktopPanel::Settings) {
            self.displays.changed.notify_one();
        }
    }

    pub(crate) async fn watch_displays(self: Arc<Self>) {
        loop {
            self.displays.changed.notified().await;
            // One dock can announce several output changes in the same burst.
            tokio::time::sleep(Duration::from_millis(150)).await;
            let gate = self.displays.wait_for_gate().await;
            if !self.locking.active() {
                let _ = self.run_display(None, self.displays.managed, gate);
            }
        }
    }

    fn start_display(
        self: &Arc<Self>,
        changes: Option<Vec<displays::Setting>>,
        automatic: bool,
    ) -> Result<()> {
        self.run_display(changes, automatic, self.displays.acquire()?)
    }

    fn run_display(
        self: &Arc<Self>,
        changes: Option<Vec<displays::Setting>>,
        automatic: bool,
        gate: OwnedMutexGuard<()>,
    ) -> Result<()> {
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        self.update(|state| {
            state.displays.busy = true;
            state.displays.error.clear();
        });
        let app = self.clone();
        tokio::spawn(async move {
            let _gate = gate;
            let result = app.display_job(changes, automatic, epoch).await;
            let observed = match &result {
                Ok((_, _, Some(outputs))) => Ok(outputs.clone()),
                _ => app.displays.observe().await,
            };
            app.update(|state| {
                state.displays.busy = false;
                state.displays.preview = None;
                match observed {
                    Ok(outputs) => {
                        if state.displays.outputs != outputs {
                            state.displays.outputs = outputs;
                            state.displays.revision += 1;
                        }
                    }
                    Err(error) => {
                        state.displays.outputs.clear();
                        state.displays.revision += 1;
                        state.displays.error = error.to_string();
                    }
                }
                match result {
                    Ok((message, error, _)) => {
                        state.displays.message = message;
                        if !error.is_empty() {
                            state.displays.error = error;
                        }
                    }
                    Err(error) => state.displays.error = error.to_string(),
                }
            });
        });
        Ok(())
    }

    async fn display_job(
        &self,
        changes: Option<Vec<displays::Setting>>,
        automatic: bool,
        epoch: u64,
    ) -> Result<(String, String, Option<Vec<displays::Output>>)> {
        let outputs = self.displays.observe().await?;
        self.check_epoch(epoch)?;
        let plan = if let Some(changes) = changes {
            ensure!(
                outputs == self.state().displays.outputs,
                "Outputs changed. Review the layout again."
            );
            Some(self.displays.prepare(&outputs, changes)?)
        } else if automatic {
            self.displays.remembered(&outputs)?
        } else {
            None
        };
        let Some(plan) = plan else {
            return Ok((
                "Connected outputs refreshed.".into(),
                String::new(),
                Some(outputs),
            ));
        };
        let token = self
            .sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (message, error) = self
            .displays
            .supervise(
                plan,
                token,
                automatic,
                |preview, choice| async move {
                    if automatic {
                        self.check_epoch(epoch).is_ok()
                    } else {
                        self.review_display(preview, choice, epoch).await
                    }
                },
                self.locking.invalidated(epoch),
            )
            .await?;
        Ok((message, error, None))
    }

    async fn review_display(
        &self,
        preview: displays::Preview,
        choice: oneshot::Receiver<bool>,
        epoch: u64,
    ) -> bool {
        self.update(|state| {
            state.displays.message = "Confirm that you can see the display.".into();
            state.displays.preview = Some(preview.clone());
        });
        let mut state = self.subscribe();
        let closed = async {
            loop {
                if state.borrow_and_update().desktop_panel != Some(DesktopPanel::Settings) {
                    return;
                }
                if state.changed().await.is_err() {
                    return;
                }
            }
        };
        let wait = Duration::from_millis(preview.expires_at.saturating_sub(displays::now_ms()));
        tokio::select! {
            biased;
            _ = self.locking.invalidated(epoch) => false,
            _ = closed => false,
            _ = tokio::time::sleep(wait) => false,
            response = choice => response.unwrap_or(false),
        }
    }
}
