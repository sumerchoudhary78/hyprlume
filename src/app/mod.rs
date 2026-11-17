//! Application state and operation routing, independent of the Unix transport.
mod commands;
mod events;
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
    pub(crate) clipboard: Arc<crate::services::clipboard::Controller>,
    pub(crate) capture: Arc<crate::services::capture::Controller>,
    pub(crate) power: Arc<crate::services::power::Controller>,
    state: watch::Sender<State>,
    pending: Mutex<Option<commands::Pending>>,
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
        let mut state = State::default();
        if demo {
            state.message = "Demo session · desktop actions are simulated".into();
        }
        let (state, _) = watch::channel(state);
        Arc::new(Self {
            config,
            desktop: Desktop::new(demo),
            connectivity: crate::services::connectivity::Controller::new(demo),
            notifications: crate::services::notifications::Controller::new(demo),
            power: crate::services::power::Controller::new(demo),
            clipboard: crate::services::clipboard::Controller::new(socket, demo),
            capture: crate::services::capture::Controller::new(demo),
            state,
            pending: Mutex::new(None),
            busy: Arc::new(Semaphore::new(1)),
            sequence: AtomicU64::new(1),
            #[cfg(feature = "voice")]
            audio_paused: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "voice")]
            speaker: Arc::new(std::sync::Mutex::new(None)),
            #[cfg(feature = "voice")]
            voice: Mutex::new(None),
        })
    }

    pub fn update(&self, f: impl FnOnce(&mut State)) {
        self.state.send_modify(f);
    }

    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }

    pub async fn handle(self: &Arc<Self>, operation: Operation) -> Result<Value> {
        match operation {
            Operation::Status | Operation::Subscribe => Ok(json!(self.state())),
            Operation::ConnectivityStatus | Operation::ConnectivitySubscribe => {
                Ok(json!(self.connectivity.state()))
            }
            Operation::NotificationsStatus | Operation::NotificationsSubscribe => {
                Ok(json!(self.notifications.state()))
            }
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
