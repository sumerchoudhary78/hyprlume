use super::App;
use crate::{protocol::DesktopPanel, services::storage::Request};
use anyhow::{ensure, Result};
use serde_json::Value;
use std::sync::Arc;
#[cfg(target_os = "linux")]
use {
    crate::{
        protocol::State as DesktopState,
        services::storage::{native, State},
    },
    anyhow::Context,
    serde_json::json,
    std::time::Duration,
    tokio::sync::watch,
};

impl App {
    pub(super) fn storage(self: &Arc<Self>, request: Request) -> Result<Value> {
        ensure!(
            !self.desktop.demo,
            "Drive actions are disabled in demo mode"
        );
        ensure!(
            self.state().desktop_panel == Some(DesktopPanel::Storage),
            "Open the drive panel before changing storage"
        );
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        #[cfg(not(target_os = "linux"))]
        {
            let _ = request;
            anyhow::bail!("Removable storage requires Linux and UDisks2")
        }
        #[cfg(target_os = "linux")]
        {
            let permit = self
                .storage_gate
                .clone()
                .try_lock_owned()
                .context("A drive operation is already running")?;
            self.update(|state| {
                state.storage.busy = true;
                state.storage.error.clear();
            });
            let app = self.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let handoff = matches!(request, Request::Open { .. });
                let result = async {
                    let connection = native::connect()
                        .await
                        .context("Connect to the system storage service")?;
                    let action = native::apply(&connection, request).await;
                    let status = if handoff {
                        None
                    } else {
                        tokio::time::timeout(Duration::from_secs(3), native::read(&connection))
                            .await
                            .ok()
                            .and_then(Result::ok)
                    };
                    Ok::<_, anyhow::Error>((action, status))
                };
                let result = tokio::select! {
                    biased;
                    _ = app.locking.invalidated(epoch) => None,
                    result = tokio::time::timeout(Duration::from_secs(45), result) => Some(result),
                };
                app.update(|state| {
                    let Some(result) = result.filter(|_| app.check_epoch(epoch).is_ok()) else {
                        state.storage = State::default();
                        return;
                    };
                    match result {
                        Ok(Ok((action, status))) => {
                            if let Some(status) = status {
                                state.storage = status;
                            }
                            match action {
                                Ok(message) => {
                                    state.storage.message = message.into();
                                    if handoff && state.desktop_panel == Some(DesktopPanel::Storage)
                                    {
                                        state.desktop_panel = None;
                                    }
                                }
                                Err(error) => state.storage.error = error.to_string(),
                            }
                        }
                        Ok(Err(error)) => state.storage.error = error.to_string(),
                        Err(_) => {
                            state.storage.error =
                                "Drive request timed out. Refresh before retrying.".into()
                        }
                    }
                    state.storage.busy = false;
                });
            });
            Ok(json!({"accepted":true}))
        }
    }

    #[cfg(target_os = "linux")]
    pub(crate) async fn watch_storage(self: Arc<Self>) {
        let mut state = self.subscribe();
        loop {
            if state.borrow().desktop_panel == Some(DesktopPanel::Storage) {
                if !self.desktop.demo {
                    tokio::select! {
                        biased;
                        _ = panel_closed(&mut state) => continue,
                        result = self.storage_events() => {
                            if result.is_err() {
                                self.report_storage(State { error: "Could not observe the system storage service.".into(), ..State::default() });
                            }
                        }
                    }
                }
                panel_closed(&mut state).await;
            }
            if state.changed().await.is_err() {
                return;
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn report_storage(&self, snapshot: State) {
        self.state.send_if_modified(|state| {
            if state.desktop_panel != Some(DesktopPanel::Storage) || state.storage.busy {
                return false;
            }
            // Preserve the most recent action result across its own D-Bus signals.
            let next = State {
                message: if state.storage.available && snapshot.available {
                    state.storage.message.clone()
                } else {
                    snapshot.message
                },
                error: if snapshot.available {
                    state.storage.error.clone()
                } else {
                    snapshot.error
                },
                ..snapshot
            };
            if state.storage == next {
                return false;
            }
            state.storage = next;
            true
        });
    }

    #[cfg(target_os = "linux")]
    async fn storage_events(&self) -> Result<()> {
        let connection = tokio::time::timeout(Duration::from_secs(3), native::connect()).await??;
        let (mut signals, mut owners) =
            tokio::time::timeout(Duration::from_secs(3), native::watch(&connection)).await??;
        loop {
            let result = tokio::time::timeout(Duration::from_secs(3), native::read(&connection))
                .await
                .context("Drive status timed out")?;
            self.report_storage(match result {
                Ok(status) => status,
                Err(error) => State {
                    error: error.to_string(),
                    ..State::default()
                },
            });
            native::changes(&mut signals, &mut owners).await?;
            tokio::time::sleep(Duration::from_millis(125)).await;
        }
    }
}

#[cfg(target_os = "linux")]
async fn panel_closed(state: &mut watch::Receiver<DesktopState>) {
    while state.borrow_and_update().desktop_panel == Some(DesktopPanel::Storage) {
        if state.changed().await.is_err() {
            return;
        }
    }
}
