use super::App;
use crate::services::credentials::{self, Request};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::sync::Arc;

impl App {
    pub(super) fn credentials(self: &Arc<Self>, request: Request) -> Result<Value> {
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        ensure!(
            !self.desktop.demo,
            "Credential controls are disabled in demo mode"
        );
        let permit = self
            .credentials_gate
            .clone()
            .try_lock_owned()
            .context("A credential action is already running")?;
        self.update(|state| {
            state.credentials.busy = true;
            state.credentials.error.clear();
        });
        let app = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let result = credentials::handle(&app.locking, epoch, request).await;
            app.update(|state| {
                if app.check_epoch(epoch).is_err() {
                    state.credentials = credentials::State::default();
                } else {
                    match result {
                        Ok(status) => state.credentials = status,
                        Err(_) => {
                            state.credentials.busy = false;
                            state.credentials.error = "Could not complete the credential request. Refresh to check the current provider.".into();
                        }
                    }
                }
            });
        });
        Ok(json!({"accepted":true}))
    }
}
