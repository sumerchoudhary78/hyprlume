use super::App;
use crate::services::{idle::Event, power::DimLease};
use anyhow::{ensure, Result};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct Effects {
    brightness: DimLease,
    displays: Vec<crate::desktop::Display>,
}
impl App {
    fn idle_allowed(&self, epoch: u64) -> bool {
        self.idle.current(epoch)
            && self.settings.state().values.preferences.idle.enabled
            && !self.idle.state().presentation
            && !self.capture.state().recording
    }
    pub(crate) async fn run_idle(self: Arc<Self>) {
        self.idle
            .clone()
            .run(self.settings.subscribe(), self.capture.subscribe())
            .await;
    }
    pub(crate) async fn idle_actions(self: Arc<Self>) {
        let mut actions = self.idle.receiver();
        loop {
            let action = tokio::select! {
                _ = self.idle.restore.notified() => { self.restore_idle().await; continue; },
                action = actions.recv() => { let Some(action) = action else {break;}; action }
            };
            if !self.idle_allowed(action.epoch) {
                continue;
            }
            let mut effects = self.idle_effects.lock().await;
            let result: Result<()> = async {
                match action.event {
                    Event::Dim => {
                        self.power.dim(self.settings.state().values.preferences.idle.dim_percent, &mut effects.brightness).await?;
                    }
                    Event::Lock | Event::Off | Event::Suspend => {
                        // Cancellation during lock preparation prevents a late
                        // display-off/suspend after activity or a policy change.
                        tokio::select! {
                            _ = self.idle.cancelled(action.epoch) => return Ok(()),
                            ready = async {self.lock().await?; self.locking.wait_ready().await} => ready?,
                        }
                        ensure!(self.idle_allowed(action.epoch), "Idle action cancelled by activity or an inhibitor");
                        if action.event == Event::Off {
                            ensure!(self.locking.state().phase == "locked", "Display off requires a confirmed lock");
                            self.desktop.blank_displays(&mut effects.displays).await?;
                        } else if action.event == Event::Suspend {
                            tokio::select! {
                                _ = self.idle.cancelled(action.epoch) => return Ok(()),
                                result = self.power.idle_suspend() => result?,
                            }
                        }
                    }
                    _ => unreachable!("Only idle stages enter the executor"),
                }
                Ok(())
            }.await;
            let dimmed = effects.brightness.active();
            let display_off = !effects.displays.is_empty();
            drop(effects);
            self.idle.update(|s| {
                s.dimmed = dimmed;
                s.display_off = display_off;
                if result.is_ok() {
                    s.phase = format!("{:?}", action.event).to_lowercase();
                    s.error.clear();
                }
            });
            if let Err(error) = result {
                self.idle.failure(&error);
            }
            if !self.idle.current(action.epoch) {
                self.restore_idle().await;
            }
        }
    }
    pub(crate) async fn restore_idle(&self) {
        let mut effects = self.idle_effects.lock().await;
        let display = self.desktop.restore_displays(&mut effects.displays).await;
        let brightness = self.power.restore_brightness(&mut effects.brightness).await;
        self.idle.update(|s| {
            s.dimmed = effects.brightness.active();
            s.display_off = !effects.displays.is_empty();
            s.phase = if s.presentation || s.recording_inhibits {
                "inhibited"
            } else if s.enabled {
                "watching"
            } else {
                "disabled"
            }
            .into();
        });
        if let Err(error) = display.and(brightness) {
            self.idle.failure(&error);
        }
    }
}
