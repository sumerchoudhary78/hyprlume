//! Event-driven compositor observations, independent of socket sessions.
use super::App;
use std::{sync::Arc, time::Duration};

impl App {
    pub(crate) async fn watch_desktop(self: Arc<Self>) {
        let Some(conn) = self.desktop.connection.clone() else {
            return;
        };
        loop {
            let result = async {
                let mut stream = hyprpilot_core::EventStream::connect(&conn).await?;
                if let Ok(ws) = conn.active_workspace().await {
                    self.update(|s| s.workspace = Some(ws.id));
                }
                if let Ok(Some(window)) = conn.active_window().await {
                    self.update(|s| s.active_app = window.class);
                }
                while let Some(event) = stream.next().await? {
                    let event = hyprpilot_core::Event::from_raw(event);
                    match event.kind.as_str() {
                        "workspacev2" => {
                            if let Some(id) = event.field("id").and_then(|v| v.parse().ok()) {
                                self.update(|s| s.workspace = Some(id));
                            }
                        }
                        "activewindow" => {
                            if let Some(class) = event.field("class") {
                                self.update(|s| s.active_app = class.chars().take(64).collect());
                            }
                        }
                        _ => {}
                    }
                }
                Ok::<_, anyhow::Error>(())
            }
            .await;
            tracing::debug!(?result, "Hyprland event stream disconnected");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
