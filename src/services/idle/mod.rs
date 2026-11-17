//! Bounded, opt-in idle policy. No polling, user shell commands or model decisions.
mod provider;
#[cfg(test)]
mod tests;
mod types;
use crate::services::{capture, settings};
use anyhow::{ensure, Context, Result};
pub use provider::callback;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{mpsc, watch, Notify};
pub use types::{Credential, Event, Request, State};

#[derive(Default)]
struct Runtime {
    token: Option<Credential>,
    epoch: u64,
    idle: bool,
    seen: u8,
    timers: bool,
}
#[derive(Clone, Copy)]
pub(crate) struct Action {
    pub event: Event,
    pub epoch: u64,
}
pub struct Controller {
    state: watch::Sender<State>,
    runtime: Mutex<Runtime>,
    epoch: watch::Sender<u64>,
    actions: mpsc::Sender<Action>,
    receiver: Mutex<Option<mpsc::Receiver<Action>>>,
    pub(crate) restore: Notify,
    restart: Notify,
    socket: PathBuf,
    demo: bool,
}
impl Controller {
    pub fn new(socket: PathBuf, demo: bool) -> Arc<Self> {
        let (state, _) = watch::channel(State {
            phase: "disabled".into(),
            message: "Automatic idle actions are off".into(),
            ..State::default()
        });
        let (epoch, _) = watch::channel(0);
        let (actions, receiver) = mpsc::channel(8);
        Arc::new(Self {
            state,
            runtime: Mutex::new(Runtime::default()),
            epoch,
            actions,
            receiver: Mutex::new(Some(receiver)),
            restore: Notify::new(),
            restart: Notify::new(),
            socket,
            demo,
        })
    }
    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }
    pub(crate) fn receiver(&self) -> mpsc::Receiver<Action> {
        self.receiver
            .lock()
            .unwrap()
            .take()
            .expect("one idle executor")
    }
    pub(crate) fn current(&self, epoch: u64) -> bool {
        *self.epoch.borrow() == epoch
    }
    pub(crate) async fn cancelled(&self, epoch: u64) {
        let mut changes = self.epoch.subscribe();
        while *changes.borrow_and_update() == epoch {
            if changes.changed().await.is_err() {
                break;
            }
        }
    }
    pub(crate) fn update(&self, f: impl FnOnce(&mut State)) {
        self.state.send_modify(f);
    }
    pub(crate) fn failure(&self, error: &anyhow::Error) {
        self.update(|s| {
            s.phase = "attention".into();
            s.error = error
                .to_string()
                .chars()
                .filter(|c| !c.is_control())
                .take(256)
                .collect();
        });
    }
    fn reset(&self, revoke: bool) {
        let mut runtime = self.runtime.lock().unwrap();
        runtime.epoch += 1;
        runtime.idle = false;
        runtime.seen = 0;
        if revoke {
            runtime.token = None;
            runtime.timers = false;
        }
        self.epoch.send_replace(runtime.epoch);
        self.restore.notify_one();
    }
    pub(crate) fn shutdown(&self) {
        self.reset(true);
        self.restart.notify_one();
    }
    pub(crate) fn handle(&self, request: Request) -> State {
        let Request::Presentation { enabled } = request;
        if self.state().presentation != enabled {
            self.update(|s| {
                s.presentation = enabled;
            });
            self.reset(true);
            self.restart.notify_one();
        }
        self.state()
    }
    /// Returns true only for a verified immediate logind lock/sleep hook. These
    /// bypass the action queue so PrepareForSleep cannot deadlock behind Suspend.
    pub(crate) fn event(&self, credential: Credential, event: Event) -> Result<bool> {
        let mut runtime = self.runtime.lock().unwrap();
        ensure!(
            runtime.token.as_ref().is_some_and(|t| t.0 == credential.0),
            "Stale or invalid idle callback"
        );
        match event {
            Event::BeforeSleep | Event::ForceLock => return Ok(true),
            Event::Resume | Event::AfterResume => {
                if runtime.idle || event == Event::AfterResume {
                    runtime.epoch += 1;
                    runtime.idle = false;
                    runtime.seen = 0;
                    self.epoch.send_replace(runtime.epoch);
                    self.restore.notify_one();
                }
                return Ok(false);
            }
            _ => (),
        }
        ensure!(runtime.timers, "Idle timers are inhibited");
        if runtime.seen & event.bit() != 0 {
            return Ok(false);
        }
        self.actions
            .try_send(Action {
                event,
                epoch: runtime.epoch,
            })
            .context("Idle action queue is full")?;
        runtime.seen |= event.bit();
        runtime.idle = true;
        Ok(false)
    }
    pub(crate) async fn run(
        self: Arc<Self>,
        mut settings: watch::Receiver<settings::State>,
        mut capture: watch::Receiver<capture::State>,
    ) {
        loop {
            let policy = settings.borrow_and_update().values.preferences.idle.clone();
            let recording = capture.borrow_and_update().recording;
            let presentation = self.state().presentation;
            let paused = recording || presentation;
            self.update(|s| {
                s.enabled = policy.enabled;
                s.recording_inhibits = recording;
            });
            if !policy.enabled || self.demo {
                self.update(|s| {
                    s.available = false;
                    s.phase = "disabled".into();
                    s.message = if self.demo {
                        "Idle actions are disabled in demo mode"
                    } else {
                        "Automatic idle actions are off"
                    }
                    .into();
                });
                tokio::select! { _ = settings.changed() => (), _ = capture.changed() => (), _ = self.restart.notified() => () }
                continue;
            }
            let result = provider::credential().and_then(|token| {
                // Publish the capability before spawning: a sleep signal can arrive immediately.
                {
                    let mut runtime = self.runtime.lock().unwrap();
                    runtime.token = Some(token.clone());
                    runtime.timers = !paused;
                }
                provider::start(&self.socket, &token, &policy, paused)
            });
            match result {
                Err(error) => {
                    self.reset(true);
                    self.update(|s| {
                        s.available = false;
                    });
                    self.failure(&error);
                    tokio::select! { _ = tokio::time::sleep(Duration::from_secs(5)) => (),
                    _ = settings.changed() => (), _ = capture.changed() => (), _ = self.restart.notified() => () }
                }
                Ok(mut child) => {
                    self.update(|s| {
                        s.available = true;
                        s.error.clear();
                        s.phase = if paused { "inhibited" } else { "watching" }.into();
                        s.message = if paused {
                            "Idle timers paused · sleep locking remains active"
                        } else {
                            "Waiting for activity · application idle inhibitors are respected"
                        }
                        .into();
                    });
                    let exited = loop {
                        tokio::select! {
                            result = child.wait() => { break Some(result); },
                            _ = settings.changed() => (),
                            _ = capture.changed() => (),
                            _ = self.restart.notified() => (),
                        }
                        if settings.borrow().values.preferences.idle != policy
                            || capture.borrow().recording != recording
                            || self.state().presentation != presentation
                            || self.runtime.lock().unwrap().token.is_none()
                        {
                            break None;
                        }
                        settings.borrow_and_update();
                        capture.borrow_and_update();
                    };
                    self.reset(true);
                    self.update(|s| {
                        s.available = false;
                    });
                    if exited.is_none() {
                        // No callbacks from the old process may mutate the new policy.
                        if let Some(pid) = child.id() {
                            let _ = rustix::process::kill_process(
                                rustix::process::Pid::from_raw(pid as i32).unwrap(),
                                rustix::process::Signal::TERM,
                            );
                        }
                        if tokio::time::timeout(Duration::from_secs(1), child.wait())
                            .await
                            .is_err()
                        {
                            let _ = child.kill().await;
                        }
                    } else {
                        self.failure(&anyhow::anyhow!(
                            "Idle provider stopped; retrying in five seconds"
                        ));
                        tokio::select! { _ = tokio::time::sleep(Duration::from_secs(5)) => (), _ = self.restart.notified() => (), _ = settings.changed() => (), _ = capture.changed() => () }
                    }
                }
            }
        }
    }
}
