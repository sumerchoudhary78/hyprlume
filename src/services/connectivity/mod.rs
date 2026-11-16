//! NetworkManager/BlueZ control and a separate, bounded UI event topic.
#[cfg(target_os = "linux")]
mod native;
mod pairing;
#[cfg(test)]
mod tests;
mod types;
pub use types::*;

use anyhow::{Context, Result};
use futures_util::future::BoxFuture;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::Duration;
use tokio::sync::{watch, Mutex, Semaphore};

pub(crate) trait Backend: Send + Sync {
    fn snapshot(&self) -> BoxFuture<'_, Result<Hardware>>;
    fn execute(&self, request: Request, state: State) -> BoxFuture<'_, Result<()>>;
    fn stop_discovery(&self) -> BoxFuture<'_, ()>;
}

pub struct Controller {
    state: watch::Sender<State>,
    backend: OnceLock<Arc<dyn Backend>>,
    busy: Arc<Semaphore>,
    refreshing: Mutex<()>,
    pairing: Arc<pairing::Pairing>,
    pair_cancel: StdMutex<Option<tokio::sync::oneshot::Sender<()>>>,
    demo: bool,
}

impl Controller {
    pub fn new(demo: bool) -> Arc<Self> {
        let (state, _) = watch::channel(State {
            message: "Connecting to desktop services".into(),
            ..State::default()
        });
        Arc::new(Self {
            pairing: pairing::Pairing::new(state.clone(), Duration::from_secs(45)),
            state,
            backend: OnceLock::new(),
            busy: Arc::new(Semaphore::new(1)),
            refreshing: Mutex::new(()),
            demo,
            pair_cancel: StdMutex::new(None),
        })
    }

    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub(crate) fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }

    pub(crate) async fn close(&self) {
        if let Some(cancel) = self.pair_cancel.lock().unwrap().take() {
            let _ = cancel.send(());
        }
        self.pairing.cancel();
        self.state.send_modify(|state| state.panel_open = false);
        if let Some(backend) = self.backend.get() {
            backend.stop_discovery().await;
        }
    }

    pub(crate) async fn handle(self: &Arc<Self>, request: Request) -> Result<Value> {
        request.validate(&self.state())?;
        match request {
            Request::Toggle => {
                if self.state().panel_open {
                    self.close().await;
                } else {
                    self.state.send_modify(|state| state.panel_open = true);
                }
                Ok(json!(self.state()))
            }
            Request::Close => {
                self.close().await;
                Ok(json!(self.state()))
            }
            Request::PairReply {
                token,
                accept,
                value,
            } => {
                self.pairing.reply(token, accept, value)?;
                Ok(json!({"accepted":true}))
            }
            request => {
                let backend = self
                    .backend
                    .get()
                    .context("Desktop connectivity services are not ready")?
                    .clone();
                let permit = self
                    .busy
                    .clone()
                    .try_acquire_owned()
                    .context("A connection operation is already running")?;
                let state = self.state();
                let pairing = matches!(request, Request::BluetoothPair { .. });
                let discovery = matches!(request, Request::BluetoothScan { .. });
                let cancel = if pairing {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    *self.pair_cancel.lock().unwrap() = Some(tx);
                    Some(rx)
                } else {
                    None
                };
                let credential = match &request {
                    Request::WifiConnect { password, .. } => password.clone(),
                    _ => None,
                };
                self.state.send_modify(|state| {
                    state.busy = true;
                    if pairing {
                        state.panel_open = true;
                    }
                    state.error.clear();
                    state.message = "Updating connection".into();
                });
                let controller = self.clone();
                tokio::spawn(async move {
                    let operation = async {
                        let execution = backend.execute(request, state);
                        if let Some(cancel) = cancel {
                            tokio::select! {
                                biased;
                                _ = cancel => anyhow::bail!("Pairing cancelled"),
                                result = execution => result,
                            }
                        } else {
                            execution.await
                        }
                    };
                    let result = tokio::time::timeout(Duration::from_secs(90), operation).await;
                    let error = match result {
                        Ok(Ok(())) => None,
                        Ok(Err(error)) => Some(error.to_string()),
                        Err(_) => Some("Connection operation timed out".into()),
                    };
                    if discovery && !controller.state().panel_open {
                        backend.stop_discovery().await;
                    }
                    controller.refresh().await;
                    if pairing {
                        controller.pair_cancel.lock().unwrap().take();
                    }
                    controller.state.send_modify(|state| {
                        state.busy = false;
                        if let Some(mut error) = error {
                            if let Some(secret) =
                                credential.filter(|value| !value.expose().is_empty())
                            {
                                error = error.replace(secret.expose(), "[credential]");
                            }
                            state.error = error.chars().take(256).collect();
                            state.message = "Connection needs attention".into();
                        } else {
                            state.message = "Connection updated".into();
                        }
                    });
                    drop(permit);
                });
                Ok(json!({"accepted":true}))
            }
        }
    }

    async fn refresh(&self) {
        let Ok(_guard) = self.refreshing.try_lock() else {
            return;
        };
        if let Some(backend) = self.backend.get() {
            match tokio::time::timeout(Duration::from_secs(15), backend.snapshot()).await {
                Ok(Ok(hardware)) => self
                    .state
                    .send_modify(|state| state.hardware = hardware.bounded()),
                _ => self.state.send_modify(|state| {
                    state.error = "Could not refresh connectivity services".into()
                }),
            }
        }
    }

    pub(crate) async fn run(self: Arc<Self>) {
        if self.demo {
            self.state.send_modify(|state| {
                state.message = "Connectivity is unavailable in demo mode".into()
            });
            return;
        }
        #[cfg(not(target_os = "linux"))]
        self.state.send_modify(|state| {
            state.error = "Connectivity requires Linux, NetworkManager and BlueZ".into()
        });
        #[cfg(target_os = "linux")]
        {
            let (changes, mut events) = tokio::sync::mpsc::channel(1);
            match native::Native::new(changes, self.pairing.clone()).await {
                Ok(backend) => {
                    let _ = self.backend.set(Arc::new(backend));
                }
                Err(_) => {
                    self.state
                        .send_modify(|state| state.error = "System D-Bus is unavailable".into());
                    return;
                }
            }
            self.refresh().await;
            self.state
                .send_modify(|state| state.message = "Your wireless connections".into());
            while events.recv().await.is_some() {
                tokio::time::sleep(Duration::from_millis(200)).await;
                while events.try_recv().is_ok() {}
                self.refresh().await;
            }
        }
    }
}
