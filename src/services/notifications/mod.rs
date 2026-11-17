//! Session notification service and private UI topic, independent of AI plans.
#[cfg(all(test, target_os = "linux"))]
mod bus_tests;
#[cfg(target_os = "linux")]
mod native;
mod store;
#[cfg(test)]
mod tests;
mod types;
pub use types::{Action, Notice, Request, State};

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use store::Store;
use tokio::sync::{watch, Notify};
use types::Signal;

pub struct Controller {
    store: Mutex<Store>,
    state: watch::Sender<State>,
    changed: Notify,
    demo: bool,
    #[cfg(target_os = "linux")]
    bus: Mutex<Option<zbus::Connection>>,
}

impl Controller {
    pub fn new(demo: bool) -> Arc<Self> {
        let (state, _) = watch::channel(State::default());
        Arc::new(Self {
            store: Mutex::new(Store::default()),
            state,
            changed: Notify::new(),
            demo,
            #[cfg(target_os = "linux")]
            bus: Mutex::new(None),
        })
    }

    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub(crate) fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }

    fn change<T>(&self, update: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
        let mut store = self.store.lock().unwrap();
        let result = update(&mut store)?;
        self.state.send_replace(store.snapshot());
        self.changed.notify_one();
        Ok(result)
    }

    pub(crate) fn close(&self) {
        let _ = self.change(|store| store.request(Request::Close));
    }

    pub(crate) async fn handle(&self, request: Request) -> Result<Value> {
        if matches!(request, Request::Invoke { .. }) {
            anyhow::ensure!(
                self.state().available,
                "Notification service is unavailable"
            );
        }
        let signals = self.change(|store| store.request(request))?;
        self.emit(signals).await?;
        Ok(json!(self.state()))
    }

    async fn emit(&self, signals: Vec<Signal>) -> Result<()> {
        #[cfg(target_os = "linux")]
        if !signals.is_empty() {
            let connection = self.bus.lock().unwrap().clone();
            if let Some(connection) = connection {
                tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    native::emit(&connection, signals),
                )
                .await??;
            }
        }
        #[cfg(not(target_os = "linux"))]
        let _ = signals;
        Ok(())
    }

    fn unavailable(&self, message: &str) {
        let _ = self.change(|store| {
            store.offline(message);
            Ok(())
        });
    }

    pub(crate) async fn run(self: Arc<Self>) {
        if self.demo {
            self.unavailable("Notification delivery is disabled in demo mode");
            return;
        }
        #[cfg(not(target_os = "linux"))]
        self.unavailable("Desktop notifications require a Linux session bus");
        #[cfg(target_os = "linux")]
        if let Err(error) = native::run(self.clone()).await {
            tracing::debug!(%error, "notification service unavailable");
            self.unavailable("Could not connect to the session notification service");
        }
    }

    #[cfg(target_os = "linux")]
    async fn expire_loop(&self) {
        loop {
            let deadline = self.store.lock().unwrap().deadline();
            let timer = async {
                if let Some(deadline) = deadline {
                    tokio::time::sleep_until(deadline.into()).await;
                } else {
                    std::future::pending::<()>().await;
                }
            };
            tokio::select! {
                _ = self.changed.notified() => {},
                _ = timer => {
                    if let Ok(signals) = self.change(|store| Ok(store.expire(std::time::Instant::now()))) {
                        let _ = self.emit(signals).await;
                    }
                }
            }
        }
    }
}
