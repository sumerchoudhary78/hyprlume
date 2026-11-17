//! Explicit, owned screenshot/recording jobs. No screen reads happen at startup.
mod geometry;
mod storage;
use anyhow::{ensure, Context, Result};
use geometry::validate_geometry;
use serde::{Deserialize, Serialize};
use std::{process::Stdio, sync::Arc};
use storage::output_file;
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{watch, Semaphore},
    time::{timeout, Duration},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub screenshot_available: bool,
    pub recording_available: bool,
    pub region_available: bool,
    pub busy: bool,
    pub recording: bool,
    pub output: String,
    pub message: String,
    pub error: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Screenshot { region: bool },
    Record { region: bool },
    Stop,
}

pub struct Controller {
    state: watch::Sender<State>,
    stop: watch::Sender<bool>,
    admission: Arc<Semaphore>,
    demo: bool,
}
impl Controller {
    pub fn new(demo: bool) -> Arc<Self> {
        let (state, _) = watch::channel(State {
            screenshot_available: super::tools::available("grim"),
            recording_available: super::tools::available("wf-recorder")
                && super::tools::available("slurp"),
            region_available: super::tools::available("slurp"),
            message: "Screen capture is idle · recordings have no audio".into(),
            ..State::default()
        });
        let (stop, _) = watch::channel(false);
        Arc::new(Self {
            state,
            stop,
            admission: Arc::new(Semaphore::new(1)),
            demo,
        })
    }
    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }
    pub fn subscribe(&self) -> watch::Receiver<State> {
        self.state.subscribe()
    }
    pub fn handle(self: &Arc<Self>, request: Request) -> Result<State> {
        if matches!(request, Request::Stop) {
            self.stop.send_replace(true);
            return Ok(self.state());
        }
        let permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .context("A capture is already in progress; stop it first")?;
        let (record, region) = match request {
            Request::Screenshot { region } => (false, region),
            Request::Record { region } => (true, region),
            Request::Stop => unreachable!(),
        };
        let state = self.state();
        ensure!(
            self.demo
                || if record {
                    state.recording_available
                } else {
                    state.screenshot_available
                },
            "Install the screen capture tool first"
        );
        ensure!(
            self.demo || !region || state.region_available,
            "Install slurp to select a region"
        );
        ensure!(
            self.demo || std::env::var_os("WAYLAND_DISPLAY").is_some(),
            "Screen capture requires a Wayland session"
        );
        self.stop.send_replace(false);
        let stop = self.stop.subscribe();
        self.state.send_modify(|s| {
            s.busy = true;
            s.error.clear();
            s.message = if region {
                "Choose a region; Escape cancels"
            } else {
                "Preparing capture"
            }
            .into();
        });
        let controller = self.clone();
        tokio::spawn(async move {
            let result = controller.capture(record, region, stop).await;
            controller.state.send_modify(|s| {
                s.busy = false;
                s.recording = false;
                match result {
                    Ok(Some(path)) => {
                        s.output = path;
                        s.message = "Capture saved locally".into();
                    }
                    Ok(None) => s.message = "Capture cancelled".into(),
                    Err(error) => {
                        s.error = error.to_string();
                        s.message = "Capture failed".into();
                    }
                }
            });
            drop(permit);
        });
        Ok(self.state())
    }
    async fn capture(
        &self,
        record: bool,
        region: bool,
        mut stop: watch::Receiver<bool>,
    ) -> Result<Option<String>> {
        if self.demo {
            return Ok(None);
        }
        // Let the shell dismiss its overlay before either the selector or screenshot.
        tokio::select! { _=tokio::time::sleep(Duration::from_millis(200)) => {}, _=stop.changed()=>return Ok(None) }
        let geometry = if region || record {
            let mut selector = Command::new("slurp");
            selector.args(["-f", "%x,%y %wx%h"]);
            if record && !region {
                selector.args(["-o", "-r"]);
            }
            let mut child = selector
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .context("Start region selection")?;
            let mut bytes = Vec::new();
            let mut reader = child
                .stdout
                .take()
                .context("Region selection output unavailable")?
                .take(129);
            let result = tokio::select! {
                result=timeout(Duration::from_secs(120),async { reader.read_to_end(&mut bytes).await?; Ok::<_,anyhow::Error>(child.wait().await?) }) => result.context("Region selection timed out")??,
                _=stop.changed()=>return Ok(None)
            };
            if !result.success() {
                return Ok(None);
            }
            let geometry = String::from_utf8(bytes)?.trim().to_owned();
            validate_geometry(&geometry)?;
            Some(geometry)
        } else {
            None
        };
        if *stop.borrow() {
            return Ok(None);
        }
        let (path, file) = output_file(record)?;
        let result = async {
            if record {
                let mut command = Command::new("wf-recorder");
                command.args(["--overwrite", "--muxer", "matroska", "--framerate", "30", "--file"]).arg(&path);
                if let Some(geometry) = &geometry { command.arg("--geometry").arg(geometry); }
                let mut child = command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                    .kill_on_drop(true).spawn().context("Start screen recorder")?;
                self.state.send_modify(|state| {
                    state.recording = true;
                    state.message = "Recording · no audio · use Stop to save".into();
                });
                let status = tokio::select! {
                    status = child.wait() => status?,
                    _ = stop.changed() => {
                        if let Some(pid) = child.id().and_then(|pid| rustix::process::Pid::from_raw(pid as i32)) {
                            rustix::process::kill_process(pid, rustix::process::Signal::INT)?;
                        }
                        timeout(Duration::from_secs(8), child.wait()).await
                            .context("Recorder did not finalize; the partial MKV is kept for recovery")??
                    }
                };
                ensure!(status.success() || *stop.borrow(), "Recorder exited unexpectedly; the partial MKV is kept for recovery");
            } else {
                let mut command = Command::new("grim");
                command.args(["-t", "png", "-l", "1"]);
                if let Some(geometry) = &geometry { command.arg("-g").arg(geometry); }
                let mut child = command.arg("-").stdin(Stdio::null()).stdout(Stdio::from(file.try_clone()?))
                    .stderr(Stdio::null()).kill_on_drop(true).spawn().context("Start screenshot capture")?;
                let status = tokio::select! {
                    result = timeout(Duration::from_secs(10), child.wait()) => result.context("Screenshot timed out")??,
                    _ = stop.changed() => return Ok(false)
                };
                ensure!(status.success(), "Screenshot tool failed");
            }
            let sync_file = file.try_clone()?;
            tokio::task::spawn_blocking(move || sync_file.sync_all()).await??;
            ensure!(file.metadata()?.len() > 0, "The capture tool produced an empty file");
            Ok::<_, anyhow::Error>(true)
        }.await;
        match result {
            Ok(true) => Ok(Some(path.to_string_lossy().into_owned())),
            Ok(false) => {
                let _ = std::fs::remove_file(&path);
                Ok(None)
            }
            Err(error) => {
                if !record || file.metadata().is_ok_and(|m| m.len() == 0) {
                    let _ = std::fs::remove_file(&path);
                }
                Err(error)
            }
        }
    }
    pub async fn shutdown(&self) {
        self.stop.send_replace(true);
        // The job holds the sole permit until its owned recorder is finalized.
        if let Ok(permit) = timeout(Duration::from_secs(10), self.admission.acquire()).await {
            drop(permit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn region_output_is_a_geometry_never_an_argument_list() {
        for valid in ["0,0 1920x1080", "-1920,-300 200x600"] {
            validate_geometry(valid).unwrap();
        }
        for invalid in [
            "",
            "0,0 1x0",
            "0,0 1x2 --file bad",
            "0,0 1x2\n",
            "0;sh,0 1x2",
            "-2147483648,0 1x2",
        ] {
            assert!(validate_geometry(invalid).is_err(), "{invalid}");
        }
    }
    #[tokio::test]
    async fn a_capture_is_serialized_and_a_demo_never_starts_tools() {
        let c = Controller::new(true);
        c.handle(Request::Record { region: false }).unwrap();
        assert!(c.handle(Request::Screenshot { region: false }).is_err());
        c.shutdown().await;
        assert!(!c.state().busy);
    }
}
