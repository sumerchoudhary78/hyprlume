use super::App;
use crate::services::desktop_preferences::{self, Request};
use anyhow::{ensure, Result};
use serde_json::{json, Value};

impl App {
    pub(super) async fn desktop_preferences(&self, request: Request) -> Result<Value> {
        ensure!(
            !self.desktop.demo,
            "System preferences are disabled in demo mode"
        );
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        let _guard = self
            .desktop_preferences_gate
            .try_lock()
            .map_err(|_| anyhow::anyhow!("A preference operation is already running"))?;
        let operation = async {
            match request {
                Request::Association { mime } => desktop_preferences::association(mime, None).await,
                Request::SetAssociation { mime, desktop_id } => {
                    desktop_preferences::association(mime, Some(desktop_id)).await
                }
                Request::StartScreenReader => {
                    desktop_preferences::start_reader().await?;
                    Ok(json!({"message":"Orca launch requested."}))
                }
                Request::WriteEnvironment { revision } => {
                    let state = self.settings.state();
                    ensure!(
                        state.revision == revision,
                        "Settings changed. Reload before writing the app profile."
                    );
                    let desktop = &state.values.preferences.desktop;
                    let locales = if desktop.locale.is_empty() {
                        String::new()
                    } else {
                        crate::desktop::run("locale", &["-a"]).await?
                    };
                    let contents = desktop
                        .environment(&locales, std::path::Path::new("/usr/share/zoneinfo"))?;
                    self.check_epoch(epoch)?;
                    let path = self
                        .settings
                        .write_desktop_profile(revision, contents)
                        .await?;
                    Ok(
                        json!({"profile":path,"message":"App profile saved. New user-service launches pick it up after sign-in."}),
                    )
                }
            }
        };
        let result = tokio::select! {
            biased;
            _ = self.locking.invalidated(epoch) => {
                self.check_epoch(epoch)?;
                unreachable!()
            }
            result = operation => result
        }?;
        self.check_epoch(epoch)?;
        Ok(result)
    }
}
