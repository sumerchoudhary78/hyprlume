//! Secure lock orchestration, separate from shell rendering and AI commands.
mod marker;
#[cfg(target_os = "linux")]
mod native;
mod worker;
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use tokio::sync::watch;
pub use worker::run as run_worker;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub available: bool,
    pub active: bool,
    pub phase: String,
    pub message: String,
}
pub struct Controller {
    state: watch::Sender<State>,
    active: AtomicBool,
    launching: AtomicBool,
    epoch: AtomicU64,
    demo: bool,
}
impl Controller {
    pub fn new(demo: bool) -> Arc<Self> {
        let check = worker::preflight();
        let active = !demo && marker::exists();
        let (state, _) = watch::channel(State {
            available: !demo && check.is_ok(),
            active,
            phase: if active { "recovery" } else { "idle" }.into(),
            message: if demo {
                "Locking is unavailable in a demo session".into()
            } else {
                check
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Ready for secure Wayland locking".into())
            },
        });
        Arc::new(Self {
            state,
            active: AtomicBool::new(active),
            launching: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            demo,
        })
    }
    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }
    pub fn active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::SeqCst)
    }
    pub(crate) async fn invalidated(&self, epoch: u64) {
        let mut state = self.subscribe();
        while self.epoch() == epoch && !self.active() {
            if state.changed().await.is_err() {
                break;
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn simulate(&self, active: bool) {
        if active {
            self.epoch.fetch_add(1, Ordering::SeqCst);
        }
        self.active.store(active, Ordering::SeqCst);
        self.state.send_modify(|state| {
            state.active = active;
            state.phase = if active { "locked" } else { "idle" }.into();
        });
    }
    pub(crate) fn reserve(&self) -> Result<()> {
        ensure!(!self.demo, "A demo session cannot lock the real desktop");
        worker::preflight()?;
        ensure!(
            !self.launching.swap(true, Ordering::SeqCst),
            "A lock request is already in progress"
        );
        if self.state().phase == "locked" {
            self.launching.store(false, Ordering::SeqCst);
            bail!("The session is already locked");
        }
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.active.store(true, Ordering::SeqCst);
        self.state.send_modify(|s| {
            s.active = true;
            s.phase = "starting".into();
            s.message = "Pausing desktop actions before locking".into();
        });
        Ok(())
    }
    pub(crate) async fn launch(self: &Arc<Self>) {
        #[cfg(target_os = "linux")]
        let result = native::start().await;
        #[cfg(not(target_os = "linux"))]
        let result: Result<()> = Err(anyhow::anyhow!("Session locking requires Linux"));
        self.launching.store(false, Ordering::SeqCst);
        match result {
            Ok(()) => {
                self.active.store(true, Ordering::SeqCst);
                self.state.send_modify(|s| {
                    s.active = true;
                    s.phase = "locked".into();
                    s.message =
                        "Compositor lock confirmed · authenticate on the lock screen".into();
                });
            }
            Err(error) => {
                let active = marker::exists();
                self.active.store(active, Ordering::SeqCst);
                self.state.send_modify(|s| {
                    s.active = active;
                    s.phase = if active { "recovery" } else { "error" }.into();
                    s.message = format!("Lock could not be confirmed: {error}");
                });
            }
        }
        #[cfg(target_os = "linux")]
        let _ =
            tokio::time::timeout(std::time::Duration::from_secs(2), native::refresh(self)).await;
    }
    pub async fn run(self: Arc<Self>) {
        #[cfg(target_os = "linux")]
        if !self.demo {
            loop {
                if let Err(error) = native::observe(self.clone()).await {
                    self.state.send_modify(|s| {
                        s.message = format!("Lock service connection unavailable: {error}");
                    });
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
    #[cfg(target_os = "linux")]
    fn observed(&self, service: &str) {
        if self.launching.load(Ordering::SeqCst) && !matches!(service, "active" | "activating") {
            return;
        }
        let marker = marker::exists();
        let active = marker
            || matches!(
                service,
                "active" | "activating" | "deactivating" | "reloading"
            );
        let previous = self.active.swap(active, Ordering::SeqCst);
        if active && !previous {
            self.epoch.fetch_add(1, Ordering::SeqCst);
        }
        if !active && self.state().phase == "error" {
            return;
        }
        let phase = if service == "active" && marker {
            "locked"
        } else if matches!(service, "activating" | "reloading" | "deactivating") {
            "starting"
        } else if marker {
            "recovery"
        } else {
            "idle"
        };
        self.state.send_if_modified(|s| {
            if s.active==active && s.phase==phase {return false;}
            s.active=active;s.phase=phase.into();s.message=match phase {"idle"=>"Lock service finished · microphone and clipboard history remain off","recovery"=>"The compositor may still be locked. Retry locking to restore authentication, or recover from an authenticated TTY.",_=>"Secure lock service is active"}.into();true
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demos_cannot_lock_and_the_client_never_accepts_password_or_unlock_arguments() {
        let controller = Controller::new(true);
        assert!(controller.reserve().is_err());
        assert!(!controller.active());
        let args = worker::client_args();
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--config", "/dev/null"]));
        assert!(args.windows(2).any(|pair| pair == ["--ready-fd", "1"]));
        assert!(!args
            .iter()
            .any(|arg| arg == "--daemonize" || arg == "--no-unlock-indicator"));
    }
}
