//! Application state and operation routing, independent of the Unix transport.
mod commands;
mod events;
mod idle;
mod locking;
#[cfg(feature = "voice")]
mod voice;

use crate::{
    config::Config,
    desktop::Desktop,
    protocol::{Operation, State},
};
use anyhow::Result;
use serde_json::{json, Value};
#[cfg(feature = "voice")]
use std::sync::atomic::AtomicBool;
use std::sync::{atomic::AtomicU64, Arc};
use tokio::sync::{watch, Mutex, Semaphore};

pub struct App {
    pub config: Config,
    pub desktop: Desktop,
    pub(crate) connectivity: Arc<crate::services::connectivity::Controller>,
    pub(crate) notifications: Arc<crate::services::notifications::Controller>,
    pub(crate) locking: Arc<crate::services::locking::Controller>,
    pub(crate) settings: Arc<crate::services::settings::Controller>,
    pub(crate) clipboard: Arc<crate::services::clipboard::Controller>,
    pub(crate) capture: Arc<crate::services::capture::Controller>,
    pub(crate) power: Arc<crate::services::power::Controller>,
    pub(crate) idle: Arc<crate::services::idle::Controller>,
    idle_effects: Mutex<idle::Effects>,
    state: watch::Sender<State>,
    pending: Mutex<Option<commands::Pending>>,
    lock_pause: Mutex<Option<u64>>,
    busy: Arc<Semaphore>,
    sequence: AtomicU64,
    #[cfg(feature = "voice")]
    audio_paused: Arc<AtomicBool>,
    #[cfg(feature = "voice")]
    speaker: Arc<std::sync::Mutex<Option<crate::voice::Speaker>>>,
    #[cfg(feature = "voice")]
    voice: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl App {
    pub fn new(config: Config, demo: bool) -> Arc<Self> {
        Self::with_socket(
            config,
            demo,
            crate::config::socket_path().unwrap_or_default(),
        )
    }
    pub fn with_socket(config: Config, demo: bool, socket: std::path::PathBuf) -> Arc<Self> {
        Self::with_paths(config, demo, socket, crate::config::config_path())
    }
    pub fn with_paths(
        config: Config,
        demo: bool,
        socket: std::path::PathBuf,
        config_path: std::path::PathBuf,
    ) -> Arc<Self> {
        let mut state = State::default();
        if demo {
            state.message = "Demo session · desktop actions are simulated".into();
        }
        let (state, _) = watch::channel(state);
        let settings = crate::services::settings::Controller::new(&config, config_path, demo);
        let app = Arc::new(Self {
            settings,
            locking: crate::services::locking::Controller::new(demo),
            config,
            desktop: Desktop::new(demo),
            connectivity: crate::services::connectivity::Controller::new(demo),
            notifications: crate::services::notifications::Controller::new(demo),
            power: crate::services::power::Controller::new(demo),
            idle: crate::services::idle::Controller::new(socket.clone(), demo),
            idle_effects: Mutex::new(idle::Effects::default()),
            clipboard: crate::services::clipboard::Controller::new(socket, demo),
            capture: crate::services::capture::Controller::new(demo),
            state,
            pending: Mutex::new(None),
            lock_pause: Mutex::new(None),
            busy: Arc::new(Semaphore::new(1)),
            sequence: AtomicU64::new(1),
            #[cfg(feature = "voice")]
            audio_paused: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "voice")]
            speaker: Arc::new(std::sync::Mutex::new(None)),
            #[cfg(feature = "voice")]
            voice: Mutex::new(None),
        });
        app.power.set_sleep_guard(Arc::new(Arc::downgrade(&app)));
        app
    }

    pub fn update(&self, f: impl FnOnce(&mut State)) {
        self.state.send_modify(|state| {
            f(state);
            if self.locking.active() {
                state.panel_open = false;
                state.desktop_panel = None;
                state.confirmation = None;
            }
        });
    }

    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }

    pub async fn handle(self: &Arc<Self>, operation: Operation) -> Result<Value> {
        if self.locking.active() {
            let allowed = matches!(
                &operation,
                Operation::Status
                    | Operation::Subscribe
                    | Operation::Lock
                    | Operation::LockStatus
                    | Operation::LockSubscribe
                    | Operation::IdleStatus
                    | Operation::IdleSubscribe
                    | Operation::IdleEvent { .. }
                    | Operation::ConnectivityStatus
                    | Operation::ConnectivitySubscribe
                    | Operation::NotificationsStatus
                    | Operation::NotificationsSubscribe
                    | Operation::PowerStatus
                    | Operation::PowerSubscribe
                    | Operation::ClipboardStatus
                    | Operation::ClipboardSubscribe
                    | Operation::CaptureStatus
                    | Operation::CaptureSubscribe
                    | Operation::SettingsStatus
                    | Operation::PortalsStatus
                    | Operation::SettingsSubscribe
                    | Operation::Voice {
                        mode: crate::protocol::VoiceMode::Off
                    }
                    | Operation::Capture {
                        request: crate::services::capture::Request::Stop
                    }
            );
            anyhow::ensure!(
                allowed,
                "Desktop actions are paused while the session is locked or awaiting recovery"
            );
        }
        match operation {
            Operation::IdleStatus | Operation::IdleSubscribe => Ok(json!(self.idle.state())),
            Operation::Idle { request } => Ok(json!(self.idle.handle(request))),
            Operation::IdleEvent { credential, event } => {
                if self.idle.event(credential, event)? {
                    self.lock().await?;
                }
                Ok(json!({"accepted":true}))
            }
            Operation::Lock => self.lock().await,
            Operation::LockStatus | Operation::LockSubscribe => Ok(json!(self.locking.state())),
            Operation::Status | Operation::Subscribe => Ok(json!(self.state())),
            Operation::ConnectivityStatus | Operation::ConnectivitySubscribe => {
                Ok(json!(self.connectivity.state()))
            }
            Operation::NotificationsStatus | Operation::NotificationsSubscribe => {
                Ok(json!(self.notifications.state()))
            }
            Operation::SettingsStatus | Operation::SettingsSubscribe => {
                Ok(json!(self.settings.state()))
            }
            Operation::PortalsStatus => Ok(json!(
                crate::services::portals::inspect(self.desktop.demo).await
            )),
            Operation::Settings { request } => Ok(json!(self.settings.handle(request).await?)),
            Operation::ClipboardStatus | Operation::ClipboardSubscribe => {
                Ok(json!(self.clipboard.state()))
            }
            Operation::Clipboard { request } => Ok(json!(self.clipboard.handle(request).await?)),
            Operation::ClipboardIngest { generation, text } => {
                self.clipboard.ingest(generation, text)?;
                Ok(json!({"accepted":true}))
            }
            Operation::CaptureStatus | Operation::CaptureSubscribe => {
                Ok(json!(self.capture.state()))
            }
            Operation::Capture { request } => {
                let state = self.capture.handle(request)?;
                self.update(|s| {
                    if s.desktop_panel == Some(crate::protocol::DesktopPanel::Capture) {
                        s.desktop_panel = None;
                    }
                });
                Ok(json!(state))
            }
            Operation::PowerStatus | Operation::PowerSubscribe => Ok(json!(self.power.state())),
            Operation::Power { request } => self.power.handle(request).await,
            Operation::LaunchDesktop { desktop_id } => Ok(
                json!({"desktop_launched":true,"message":self.desktop.launch_desktop(&desktop_id).await?}),
            ),
            Operation::FocusWindow { address } => Ok(
                json!({"window_focused":true,"message":self.desktop.focus_address(&address).await?}),
            ),
            Operation::Notifications { request } => {
                if matches!(request, crate::services::notifications::Request::Toggle)
                    && !self.notifications.state().panel_open
                {
                    self.connectivity.close().await;
                    self.power.set_active(false);
                    self.update(|state| {
                        state.panel_open = false;
                        state.desktop_panel = None;
                    });
                }
                self.notifications.handle(request).await
            }
            Operation::Connectivity { request } => {
                if matches!(request, crate::services::connectivity::Request::Toggle)
                    && !self.connectivity.state().panel_open
                {
                    self.notifications.close();
                    self.power.set_active(false);
                    self.update(|state| {
                        state.panel_open = false;
                        state.desktop_panel = None;
                    });
                }
                self.connectivity.handle(request).await
            }
            Operation::Toggle => {
                self.power.set_active(false);
                let opening = !self.state().panel_open
                    || self.connectivity.state().panel_open
                    || self.notifications.state().panel_open
                    || self.state().desktop_panel.is_some();
                if opening {
                    self.connectivity.close().await;
                    self.notifications.close();
                }
                self.update(|state| {
                    state.panel_open = opening;
                    state.desktop_panel = None;
                });
                Ok(json!(self.state()))
            }
            Operation::Panel { panel, open } => {
                if open {
                    self.connectivity.close().await;
                    self.notifications.close();
                }
                self.update(|state| {
                    if open {
                        state.panel_open = false;
                        state.desktop_panel = Some(panel);
                    } else if state.desktop_panel == Some(panel) {
                        state.desktop_panel = None;
                    }
                });
                self.power.set_active(
                    self.state().desktop_panel == Some(crate::protocol::DesktopPanel::Power),
                );
                Ok(json!(self.state()))
            }
            Operation::Cancel => {
                self.cancel().await?;
                Ok(json!(self.state()))
            }
            Operation::Voice { mode } => {
                self.set_voice(mode).await?;
                Ok(json!(self.state()))
            }
            Operation::Command { text, dry_run } => self.submit(text, dry_run).await,
            Operation::Confirm { token } => self.confirm(token).await,
        }
    }

    fn fail(&self, message: &str) {
        self.update(|state| {
            state.phase = "error".into();
            state.message = message.chars().take(512).collect();
            state.confirmation = None;
            state.panel_open = true;
        });
    }

    #[cfg(not(feature = "voice"))]
    async fn set_voice(self: &Arc<Self>, _mode: crate::protocol::VoiceMode) -> Result<()> {
        anyhow::bail!("this build has no voice feature");
    }
}
