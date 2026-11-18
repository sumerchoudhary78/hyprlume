//! On-demand display observations and supervised layout previews.
mod layout;
mod profiles;
mod worker;

use anyhow::{ensure, Context, Result};
pub(crate) use layout::Plan;
pub use layout::{Output, Setting};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{oneshot, Notify, OwnedMutexGuard},
};
pub use worker::run as run_worker;
use worker::{Job, Reply};

const PREVIEW_MS: u64 = 15_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Refresh,
    Preview {
        revision: u64,
        changes: Vec<Setting>,
    },
    Keep {
        token: u64,
    },
    Revert {
        token: u64,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct State {
    pub outputs: Vec<Output>,
    pub revision: u64,
    pub busy: bool,
    pub preview: Option<Preview>,
    pub message: String,
    pub error: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Preview {
    pub token: u64,
    pub expires_at: u64,
}

pub(crate) struct Controller {
    path: PathBuf,
    gate: Arc<tokio::sync::Mutex<()>>,
    pending: Mutex<Option<(u64, oneshot::Sender<bool>)>>,
    topology: Mutex<Option<Vec<layout::Identity>>>,
    pub(crate) managed: bool,
    pub(crate) changed: Notify,
}

impl Controller {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path: path.with_file_name("display-layouts.json"),
            gate: Arc::new(tokio::sync::Mutex::new(())),
            pending: Mutex::new(None),
            topology: Mutex::new(None),
            changed: Notify::new(),
            managed: std::env::var("HYPRLUME_SESSION_READY").as_deref() == Ok("1"),
        }
    }

    pub(crate) fn acquire(&self) -> Result<OwnedMutexGuard<()>> {
        self.gate
            .clone()
            .try_lock_owned()
            .context("A display operation is already running")
    }

    pub(crate) async fn wait_for_gate(&self) -> OwnedMutexGuard<()> {
        self.gate.clone().lock_owned().await
    }

    pub(crate) async fn observe(&self) -> Result<Vec<Output>> {
        worker::outputs().await
    }

    pub(crate) fn prepare(&self, outputs: &[Output], changes: Vec<Setting>) -> Result<Plan> {
        let plan = Plan::new(outputs, changes)?;
        ensure!(!plan.changes().is_empty(), "The layout is already active");
        Ok(plan)
    }

    pub(crate) fn remembered(&self, outputs: &[Output]) -> Result<Option<Plan>> {
        let topology = layout::snapshot(outputs).topology;
        let mut last = self.topology.lock().unwrap();
        if last.as_ref() == Some(&topology) {
            return Ok(None);
        }
        *last = Some(topology);
        profiles::plan(&self.path, outputs)
    }

    pub(crate) fn decide(&self, token: u64, keep: bool) -> Result<()> {
        let mut pending = self.pending.lock().unwrap();
        ensure!(
            pending.as_ref().is_some_and(|(id, _)| *id == token),
            "The display preview expired"
        );
        pending
            .take()
            .unwrap()
            .1
            .send(keep)
            .map_err(|_| anyhow::anyhow!("The display preview expired"))
    }

    pub(crate) async fn supervise<F, Fut, Cancel>(
        &self,
        plan: Plan,
        token: u64,
        automatic: bool,
        present: F,
        cancel: Cancel,
    ) -> Result<(String, String)>
    where
        F: FnOnce(Preview, oneshot::Receiver<bool>) -> Fut,
        Fut: std::future::Future<Output = bool>,
        Cancel: std::future::Future<Output = ()>,
    {
        let executable = std::env::current_exe()?;
        let mut command = if std::env::var("HYPRLUME_SYSTEMD_DAEMON").as_deref() == Ok("1") {
            // A separate scope can restore the outputs if the daemon is restarted.
            let mut command = Command::new("systemd-run");
            command
                .args(["--user", "--scope", "--quiet", "--"])
                .arg(executable);
            command
        } else {
            Command::new(executable)
        };
        let mut child = command
            .arg("display-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Start display rollback worker")?;
        let mut input = child.stdin.take().context("Open display worker input")?;
        let mut output = BufReader::new(child.stdout.take().context("Open display worker output")?);
        let job = Job {
            plan,
            profile: self.path.clone(),
            token,
            automatic,
        };
        let result = async {
            let body = serde_json::to_vec(&job)?;
            ensure!(body.len() < 128 * 1024 - 8, "Display plan is too large");
            input.write_all(&body).await?;
            input.write_all(b"\n").await?;
            let first = tokio::select! {
                biased;
                _ = cancel => return Err(anyhow::anyhow!("Display change cancelled")),
                reply = reply(&mut output) => reply?,
            };
            let final_reply = match first {
                Reply::Applied { preview } => {
                    let (choice, response) = oneshot::channel();
                    *self.pending.lock().unwrap() = Some((token, choice));
                    let keep = present(preview, response).await;
                    self.pending.lock().unwrap().take();
                    // A worker whose deadline elapsed may have closed its input.
                    // Its final receipt still describes the actual restored state.
                    let _ = input
                        .write_all(if keep { b"keep\n" } else { b"revert\n" })
                        .await;
                    reply(&mut output).await?
                }
                other => other,
            };
            match final_reply {
                Reply::Done { message, error } => Ok((message, error)),
                Reply::Applied { .. } => anyhow::bail!("Unexpected reply from display worker"),
            }
        }
        .await;
        // EOF requests rollback even if the worker has yet to finish applying.
        drop(input);
        let _ = tokio::time::timeout(Duration::from_secs(18), child.wait()).await;
        result
    }
}

async fn reply(output: &mut BufReader<tokio::process::ChildStdout>) -> Result<Reply> {
    let mut line = String::new();
    let bytes = tokio::time::timeout(Duration::from_secs(16), output.read_line(&mut line))
        .await
        .context("Display worker timed out")??;
    ensure!(
        bytes > 0,
        "The worker stopped before reporting the display state"
    );
    Ok(serde_json::from_str(&line)?)
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
