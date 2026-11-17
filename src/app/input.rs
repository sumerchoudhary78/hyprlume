use super::App;
use crate::{
    desktop::input::{read_catalog, Request, Status, SystemKey},
    protocol::Feedback,
};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use std::{
    sync::{atomic::Ordering, Arc},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

impl App {
    pub(super) async fn input(&self, request: Request) -> Result<Value> {
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        if matches!(request, Request::Status) {
            let status = match self.desktop.input_devices().await {
                Ok(keyboards) => Status {
                    available: true,
                    keyboards,
                    profile: String::new(),
                    message: "Input settings are ready".into(),
                },
                Err(error) => Status {
                    message: error.to_string(),
                    ..Status::default()
                },
            };
            return Ok(json!(status));
        }
        let _permit = self
            .input_gate
            .try_lock()
            .map_err(|_| anyhow::anyhow!("An input action is already running"))?;
        let action = async {
            let revision = match request {
                Request::NextLayout => {
                    let layout = self.desktop.next_layout().await?;
                    self.check_epoch(epoch)?;
                    self.feedback("Keyboard layout", layout, "keyboard", None);
                    return Ok(json!({"message":"Keyboard layout switched"}));
                }
                Request::Apply { revision } | Request::WriteProfile { revision } => revision,
                Request::Status => unreachable!(),
            };
            let state = self.settings.state();
            ensure!(
                state.revision == revision,
                "Settings changed. Reload before applying."
            );
            let preferences = &state.values.preferences;
            read_catalog(&preferences.input).await?;
            self.check_epoch(epoch)?;
            ensure!(
                self.settings.state().revision == revision,
                "Settings changed. Reload before applying."
            );
            if matches!(request, Request::Apply { .. }) {
                self.desktop.apply_input(&preferences.input).await?;
                Ok(json!({"message":"Input options applied to the running compositor"}))
            } else {
                let (name, contents) = self
                    .desktop
                    .input_profile(&preferences.input, &preferences.shortcuts)
                    .await?;
                self.check_epoch(epoch)?;
                let profile = self
                    .settings
                    .write_input_profile(revision, name, contents)
                    .await?;
                Ok(
                    json!({"message":"Profile written. Include it in your Hyprland configuration.", "profile":profile}),
                )
            }
        };
        tokio::select! {
            biased;
            _ = self.locking.invalidated(epoch) => {
                self.check_epoch(epoch)?;
                unreachable!()
            }
            result = action => result
        }
    }

    pub(super) fn system_key(self: &Arc<Self>, key: SystemKey) -> Result<Value> {
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        ensure!(!self.desktop.demo, "System keys are disabled in demo mode");
        let Ok(permit) = self.input_gate.clone().try_lock_owned() else {
            return Ok(json!({"accepted":false}));
        };
        let app = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let apply = async {
                use SystemKey::*;
                match key {
                    BrightnessUp | BrightnessDown => {
                        let step = if matches!(key, BrightnessUp) { 5 } else { -5 };
                        let value = app.power.step_brightness(step).await?;
                        Ok((format!("{value}%"), Some(value)))
                    }
                    _ => key.apply_audio().await,
                }
            };
            let (title, icon) = key.presentation();
            let result = tokio::select! {
                biased;
                _ = app.locking.invalidated(epoch) => return,
                result = tokio::time::timeout(Duration::from_secs(8), apply) => result,
            };
            if app.check_epoch(epoch).is_ok() {
                let (detail, percent) = match result {
                    Ok(Ok(value)) => value,
                    Ok(Err(error)) => {
                        tracing::debug!(%error, ?key, "system key action failed");
                        ("Control unavailable".into(), None)
                    }
                    Err(_) => ("Control timed out".into(), None),
                };
                app.feedback(title, detail, icon, percent);
            }
        });
        Ok(json!({"accepted":true}))
    }

    fn feedback(&self, title: &str, detail: String, icon: &str, percent: Option<u8>) {
        self.update(|state| {
            state.feedback = Some(Feedback {
                sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
                issued_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                title: title.into(),
                detail,
                icon: icon.into(),
                percent,
            });
        });
    }
}
