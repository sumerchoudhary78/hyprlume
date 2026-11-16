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
        let mut state = State::default();
        if demo {
            state.message = "Demo session · desktop actions are simulated".into();
        }
        let (state, _) = watch::channel(state);
        Arc::new(Self {
            config,
            desktop: Desktop::new(demo),
            connectivity: crate::services::connectivity::Controller::new(demo),
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
            Operation::Connectivity { request } => {
                if matches!(request, crate::services::connectivity::Request::Toggle)
                    && !self.connectivity.state().panel_open
                {
                    self.update(|state| state.panel_open = false);
                }
                self.connectivity.handle(request).await
            }
            Operation::Toggle => {
                if !self.state().panel_open {
                    self.connectivity.close().await;
                }
                self.update(|state| state.panel_open = !state.panel_open);
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
