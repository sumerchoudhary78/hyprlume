use super::App;
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use std::sync::Arc;
impl App {
    pub(super) async fn lock(self: &Arc<Self>) -> Result<Value> {
        if matches!(self.locking.state().phase.as_str(), "locked" | "starting") {
            return Ok(json!({"accepted":true}));
        }
        self.locking.reserve()?;
        self.pending.lock().await.take();
        self.update(|state| {
            state.confirmation = None;
            state.actions.clear();
            state.panel_open = false;
            state.desktop_panel = None;
        });
        let app = self.clone();
        tokio::spawn(async move {
            app.pause_for_lock().await;
            app.locking.launch().await;
        });
        Ok(json!({"accepted":true}))
    }
    async fn pause_for_lock(self: &Arc<Self>) {
        let mut paused = self.lock_pause.lock().await;
        let epoch = self.locking.epoch();
        if !self.locking.active() || *paused == Some(epoch) {
            return;
        }
        self.pending.lock().await.take();
        self.update(|state| {
            state.confirmation = None;
            state.actions.clear();
            state.phase = "ready".into();
            state.message = "Desktop actions paused for session locking".into();
        });
        let _ = self.set_voice(crate::protocol::VoiceMode::Off).await;
        self.clipboard.shutdown().await;
        self.capture.shutdown().await;
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), self.connectivity.close())
            .await;
        self.notifications.close();
        self.power.set_active(false);
        *paused = Some(epoch);
    }
    pub(crate) async fn watch_lock(self: Arc<Self>) {
        let mut state = self.locking.subscribe();
        loop {
            if state.borrow().active {
                self.pause_for_lock().await;
            }
            if state.changed().await.is_err() {
                break;
            }
        }
    }
    pub(super) fn check_epoch(&self, epoch: u64) -> Result<()> {
        ensure!(
            !self.locking.active() && self.locking.epoch() == epoch,
            "Session locking invalidated this action; request it again after authentication"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, protocol::Operation};
    use std::time::Duration;
    #[tokio::test]
    async fn a_lock_episode_revokes_old_confirmations_and_blocks_desktop_mutations() {
        let app = App::new(Config::default(), true);
        app.handle(Operation::Command {
            text: "workspace 2".into(),
            dry_run: true,
        })
        .await
        .unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while app.state().confirmation.is_none() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let token = app.state().confirmation.unwrap();
        app.locking.simulate(true);
        assert!(app
            .handle(Operation::Confirm { token })
            .await
            .unwrap_err()
            .to_string()
            .contains("paused"));
        assert!(app
            .handle(Operation::FocusWindow {
                address: "0xa".into()
            })
            .await
            .is_err());
        assert!(app
            .handle(Operation::Capture {
                request: crate::services::capture::Request::Record {
                    region: false,
                    output: None
                }
            })
            .await
            .is_err());
        assert!(app.handle(Operation::Status).await.is_ok());
        app.locking.simulate(false);
        assert!(app
            .handle(Operation::Confirm { token })
            .await
            .unwrap_err()
            .to_string()
            .contains("invalidated"));
    }
    #[tokio::test]
    async fn locking_cancels_a_waiting_model_plan_instead_of_publishing_it_later() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut config = Config::default();
        config.llm.endpoint = Some(format!("http://{}", listener.local_addr().unwrap()));
        let app = App::new(config, true);
        app.handle(Operation::Command {
            text: "arrange my research tools".into(),
            dry_run: true,
        })
        .await
        .unwrap();
        let (_connection, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
            .await
            .unwrap()
            .unwrap();
        app.locking.simulate(true);
        tokio::time::timeout(Duration::from_secs(1), async {
            while app.busy.available_permits() == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        app.locking.simulate(false);
        assert!(app.state().confirmation.is_none());
        assert!(app.pending.lock().await.is_none());
    }
    #[cfg(feature = "voice")]
    #[tokio::test]
    async fn a_queued_voice_start_cannot_cross_a_lock_and_unlock_episode() {
        let app = App::new(Config::default(), true);
        let held = app.voice.lock().await;
        let queued = app.clone();
        let request =
            tokio::spawn(async move { queued.set_voice(crate::protocol::VoiceMode::Wake).await });
        tokio::task::yield_now().await;
        app.locking.simulate(true);
        app.locking.simulate(false);
        drop(held);
        let error = tokio::time::timeout(Duration::from_secs(1), request)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert!(error.to_string().contains("invalidated"));
        assert_eq!(app.state().voice, crate::protocol::VoiceMode::Off);
        assert!(app.voice.lock().await.is_none());
    }
}
