//! Power/session service: native observations, bounded mutations and separate review.
#[cfg(all(test, target_os = "linux"))]
mod bus_tests;
mod idle;
#[cfg(target_os = "linux")]
mod native;
#[cfg(test)]
mod tests;
mod types;
pub(crate) use idle::DimLease;
pub use types::{
    Backlight, Battery, Hardware, Profile, Profiles, Prompt, Request, Session, SessionAction, State,
};

use anyhow::{ensure, Context, Result};
use futures_util::future::BoxFuture;
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{watch, Notify, Semaphore};
use types::{Mutation, SessionTarget};

pub(crate) trait Backend: Send + Sync {
    fn snapshot(&self) -> BoxFuture<'_, Result<Hardware>>;
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn backlights(&self) -> BoxFuture<'_, Result<Vec<Backlight>>>;
    fn apply(&self, mutation: Mutation, observed: Hardware) -> BoxFuture<'_, Result<String>>;
}

pub(crate) trait SleepGuard: Send + Sync {
    fn lock_before_sleep(&self) -> BoxFuture<'_, Result<()>>;
    fn locked(&self) -> bool;
}

struct Pending {
    prompt: Prompt,
    target: SessionTarget,
    deadline: tokio::time::Instant,
}

pub struct Controller {
    state: watch::Sender<State>,
    backend: Mutex<Option<Arc<dyn Backend>>>,
    sleep_guard: Mutex<Option<Arc<dyn SleepGuard>>>,
    busy: Arc<Semaphore>,
    refreshing: tokio::sync::Mutex<()>,
    pending: Mutex<Option<Pending>>,
    sequence: AtomicU64,
    active: AtomicBool,
    observers: AtomicUsize,
    visibility: AtomicU64,
    wake: Notify,
    demo: bool,
}

impl Controller {
    pub fn new(demo: bool) -> Arc<Self> {
        let (state, _) = watch::channel(State {
            message: "Connecting to power services".into(),
            ..State::default()
        });
        Arc::new(Self {
            state,
            backend: Mutex::new(None),
            sleep_guard: Mutex::new(None),
            busy: Arc::new(Semaphore::new(1)),
            refreshing: tokio::sync::Mutex::new(()),
            pending: Mutex::new(None),
            sequence: AtomicU64::new(1),
            active: AtomicBool::new(false),
            observers: AtomicUsize::new(0),
            visibility: AtomicU64::new(0),
            wake: Notify::new(),
            demo,
        })
    }
    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub(crate) fn set_sleep_guard(&self, guard: Arc<dyn SleepGuard>) {
        *self.sleep_guard.lock().unwrap() = Some(guard);
    }
    pub(crate) fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }
    pub(crate) fn observe_ui(self: &Arc<Self>) -> ViewLease {
        self.observers.fetch_add(1, Ordering::SeqCst);
        self.wake.notify_one();
        ViewLease {
            controller: self.clone(),
        }
    }

    pub(crate) fn set_active(&self, active: bool) {
        if self.active.swap(active, Ordering::SeqCst) != active {
            self.visibility.fetch_add(1, Ordering::SeqCst);
        }
        if !active {
            self.clear_prompt();
        }
        self.wake.notify_one();
    }
    fn clear_prompt(&self) {
        self.pending.lock().unwrap().take();
        self.state
            .send_if_modified(|state| state.prompt.take().is_some());
    }
    fn expire(&self) {
        let mut pending = self.pending.lock().unwrap();
        if pending
            .as_ref()
            .is_some_and(|pending| pending.deadline <= tokio::time::Instant::now())
        {
            pending.take();
            self.state.send_modify(|state| {
                state.prompt = None;
                state.message = "Session confirmation expired".into();
            });
        }
    }

    pub(crate) async fn step_brightness(&self, step: i16) -> Result<u8> {
        let _permit = self
            .busy
            .try_acquire()
            .context("A power operation is already running")?;
        let backend = self.backend()?;
        let observed = self.refresh(backend.as_ref()).await?;
        let light = observed
            .backlights
            .first()
            .context("No kernel backlight was found")?;
        let device = light.id.clone();
        let percent = (i16::from(light.percent) + step).clamp(5, 100) as u8;
        backend
            .apply(
                Mutation::Brightness {
                    device: device.clone(),
                    percent,
                },
                observed,
            )
            .await?;
        self.refresh(backend.as_ref())
            .await?
            .backlights
            .into_iter()
            .find(|light| light.id == device)
            .map(|light| light.percent)
            .context("Backlight changed or disappeared")
    }

    fn backend(&self) -> Result<Arc<dyn Backend>> {
        self.backend
            .lock()
            .unwrap()
            .clone()
            .context("Power services are unavailable")
    }
    async fn refresh(&self, backend: &dyn Backend) -> Result<Hardware> {
        let _guard = self.refreshing.lock().await;
        let hardware = tokio::time::timeout(Duration::from_secs(6), backend.snapshot())
            .await
            .context("Power service read timed out")??;
        ensure!(
            self.backend
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|current| std::ptr::addr_eq(Arc::as_ptr(current), backend)),
            "Power service connection changed"
        );
        // A replacement login/service must never inherit an old confirmation.
        let mut pending = self.pending.lock().unwrap();
        if pending
            .as_ref()
            .is_some_and(|pending| hardware.session.target.as_ref() != Some(&pending.target))
        {
            pending.take();
        }
        let prompt = pending.as_ref().map(|pending| pending.prompt.clone());
        self.state.send_if_modified(|state| {
            if state.hardware == hardware && state.prompt == prompt {
                return false;
            }
            state.hardware = hardware.clone();
            state.prompt = prompt;
            true
        });
        Ok(hardware)
    }

    pub(crate) async fn handle(self: &Arc<Self>, request: Request) -> Result<Value> {
        if let Request::Cancel { token } = request {
            let mut pending = self.pending.lock().unwrap();
            ensure!(
                pending
                    .as_ref()
                    .is_some_and(|pending| pending.prompt.token == token),
                "Session confirmation is no longer current"
            );
            pending.take();
            self.state.send_modify(|state| {
                state.prompt = None;
                state.message = "Session action cancelled".into();
            });
            self.wake.notify_one();
            return Ok(json!(self.state()));
        }
        let backend = self.backend()?;
        let permit = self
            .busy
            .clone()
            .try_acquire_owned()
            .context("A power operation is already running")?;
        if !matches!(request, Request::Refresh) {
            ensure!(
                self.active.load(Ordering::SeqCst),
                "Open the power panel before changing power settings"
            );
        }
        self.state().hardware.validate(&request)?;
        let pending = if let Request::Confirm { token } = request {
            self.expire();
            let mut pending = self.pending.lock().unwrap();
            ensure!(
                pending
                    .as_ref()
                    .is_some_and(|pending| pending.prompt.token == token),
                "Session confirmation is no longer current"
            );
            pending.take()
        } else {
            None
        };
        self.state.send_modify(|state| {
            state.busy = true;
            state.error.clear();
            if pending.is_some() {
                state.prompt = None;
            }
        });
        let controller = self.clone();
        let visibility = self.visibility.load(Ordering::SeqCst);
        tokio::spawn(async move {
            let result = controller
                .perform(request, pending, backend.as_ref(), visibility)
                .await;
            controller.state.send_modify(|state| {
                state.busy = false;
                match result {
                    Ok(message) => {
                        state.message = message;
                        state.error.clear();
                    }
                    Err(error) => {
                        state.message = "Power action needs attention".into();
                        state.error = error
                            .to_string()
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(256)
                            .collect();
                    }
                }
            });
            drop(permit);
        });
        Ok(json!({"accepted":true}))
    }

    async fn perform(
        &self,
        request: Request,
        pending: Option<Pending>,
        backend: &dyn Backend,
        visibility: u64,
    ) -> Result<String> {
        let mut observed = self.refresh(backend).await?;
        observed.validate(&request)?;
        let mutation = match request {
            Request::Refresh => return Ok("Power state refreshed".into()),
            Request::Prepare { kind } => {
                let mut pending = self.pending.lock().unwrap();
                ensure!(
                    self.active.load(Ordering::SeqCst)
                        && visibility == self.visibility.load(Ordering::SeqCst),
                    "Power panel closed before review was ready"
                );
                let prompt = Prompt {
                    token: self.sequence.fetch_add(1, Ordering::SeqCst),
                    kind,
                    session_id: observed.session.id.clone(),
                    expires_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                        + 30_000,
                };
                *pending = Some(Pending {
                    prompt: prompt.clone(),
                    target: observed
                        .session
                        .target
                        .context("Graphical session disappeared")?,
                    deadline: tokio::time::Instant::now() + Duration::from_secs(30),
                });
                self.state.send_modify(|state| {
                    state.prompt = Some(prompt);
                });
                self.wake.notify_one();
                return Ok("Review the session action".into());
            }
            Request::Brightness { device, percent } => Mutation::Brightness { device, percent },
            Request::Profile { profile } => Mutation::Profile(profile),
            Request::Confirm { .. } => {
                let pending = pending.context("Session confirmation is no longer current")?;
                ensure!(
                    pending.deadline > tokio::time::Instant::now(),
                    "Session confirmation expired"
                );
                ensure!(
                    self.active.load(Ordering::SeqCst)
                        && visibility == self.visibility.load(Ordering::SeqCst),
                    "Power panel closed before the action started"
                );
                ensure!(
                    observed.session.target.as_ref() == Some(&pending.target),
                    "Graphical session changed; review the action again"
                );
                observed.session.validate(pending.prompt.kind)?;
                Mutation::Session {
                    kind: pending.prompt.kind,
                    target: pending.target,
                }
            }
            Request::Cancel { .. } => unreachable!(),
        };
        ensure!(
            self.active.load(Ordering::SeqCst)
                && visibility == self.visibility.load(Ordering::SeqCst),
            "Power panel closed before the action started"
        );
        if let Mutation::Session {
            kind: SessionAction::Suspend,
            target,
        } = &mutation
        {
            // Locking deliberately closes the panel. The consumed confirmation is
            // bound to the verified login, rather than to visibility after locking.
            observed = self.prepare_sleep(backend, target).await?;
        }
        let result =
            tokio::time::timeout(Duration::from_secs(15), backend.apply(mutation, observed))
                .await
                .context("Power action timed out; inspect the current system state before retrying")
                .and_then(|result| result);
        // Even a failed/uncertain request may have changed observed state.
        let _ = self.refresh(backend).await;
        result
    }

    async fn prepare_sleep(
        &self,
        backend: &dyn Backend,
        target: &SessionTarget,
    ) -> Result<Hardware> {
        let before = self.refresh(backend).await?;
        ensure!(
            before.session.target.as_ref() == Some(target),
            "Graphical session changed before sleep"
        );
        ensure!(
            before.session.suspend == "yes",
            "Suspend requires authorization or is inhibited; unlock and resolve it before sleeping"
        );
        let guard = self
            .sleep_guard
            .lock()
            .unwrap()
            .clone()
            .context("Suspend requires a secure lock guard")?;
        guard.lock_before_sleep().await?;
        let current = self.refresh(backend).await?;
        ensure!(
            current.session.target.as_ref() == Some(target),
            "Graphical session changed while locking"
        );
        ensure!(
            current.session.suspend == "yes",
            "Suspend became unavailable while locking"
        );
        ensure!(
            guard.locked(),
            "Compositor lock is no longer confirmed; suspend cancelled"
        );
        Ok(current)
    }

    fn offline(&self, message: &str) {
        *self.backend.lock().unwrap() = None;
        self.clear_prompt();
        self.state.send_modify(|state| {
            state.hardware = Hardware::default();
            state.error = message.into();
            state.message = "Power services unavailable".into();
        });
    }

    pub(crate) async fn run(self: Arc<Self>) {
        if self.demo {
            self.offline("Power controls are disabled in demo mode");
            return;
        }
        #[cfg(not(target_os = "linux"))]
        self.offline("Power controls require Linux system services");
        #[cfg(target_os = "linux")]
        loop {
            if let Err(error) = native::run(self.clone()).await {
                tracing::debug!(%error, "power service disconnected");
                self.offline("Could not connect to system power services");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

/// Dropping the last subscribed client stops sampling and revokes its review.
pub(crate) struct ViewLease {
    controller: Arc<Controller>,
}
impl Drop for ViewLease {
    fn drop(&mut self) {
        if self.controller.observers.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.controller.visibility.fetch_add(1, Ordering::SeqCst);
            self.controller.clear_prompt();
        }
        self.controller.wake.notify_one();
    }
}
