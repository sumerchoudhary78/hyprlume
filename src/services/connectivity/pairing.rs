//! One bounded, single-use Bluetooth prompt. BlueZ cancellation drops its guard.
use super::{Secret, State};
use anyhow::{bail, Context, Result};
#[cfg(any(target_os = "linux", test))]
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{oneshot, watch};

struct Pending {
    token: u64,
    created: Instant,
    reply: oneshot::Sender<(bool, Option<Secret>)>,
}

pub(crate) struct Pairing {
    state: watch::Sender<State>,
    pending: Mutex<Option<Pending>>,
    #[cfg(any(target_os = "linux", test))]
    sequence: AtomicU64,
    deadline: Duration,
}

#[cfg(any(target_os = "linux", test))]
struct PromptGuard {
    pairing: Arc<Pairing>,
    token: u64,
}
#[cfg(any(target_os = "linux", test))]
impl Drop for PromptGuard {
    fn drop(&mut self) {
        let mut pending = self.pairing.pending.lock().unwrap();
        if pending.as_ref().is_some_and(|p| p.token == self.token) {
            pending.take();
        }
        self.pairing.state.send_modify(|state| {
            if state.prompt.as_ref().is_some_and(|p| p.token == self.token) {
                state.prompt = None;
            }
        });
    }
}

impl Pairing {
    pub(crate) fn new(state: watch::Sender<State>, deadline: Duration) -> Arc<Self> {
        Arc::new(Self {
            state,
            pending: Mutex::new(None),
            #[cfg(any(target_os = "linux", test))]
            sequence: AtomicU64::new(1),
            deadline,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) async fn ask(
        self: &Arc<Self>,
        adapter: String,
        address: String,
        kind: &str,
        code: String,
    ) -> Result<Option<Secret>> {
        let token = self.sequence.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().unwrap();
            anyhow::ensure!(pending.is_none(), "Another pairing prompt is already open");
            *pending = Some(Pending {
                token,
                created: Instant::now(),
                reply: tx,
            });
        }
        let _guard = PromptGuard {
            pairing: self.clone(),
            token,
        };
        self.state.send_modify(|state| {
            state.prompt = Some(super::PairPrompt {
                token,
                adapter,
                address,
                kind: kind.into(),
                code,
            });
        });
        let (accepted, value) = tokio::time::timeout(self.deadline, rx)
            .await
            .context("Pairing prompt expired")?
            .context("Pairing cancelled")?;
        anyhow::ensure!(accepted, "Pairing rejected");
        Ok(value)
    }

    pub(crate) fn reply(&self, token: u64, accepted: bool, value: Option<Secret>) -> Result<()> {
        let mut pending = self.pending.lock().unwrap();
        let prompt = pending.as_ref().context("No pairing prompt is waiting")?;
        if prompt.token != token {
            bail!("Pairing token is no longer current");
        }
        if prompt.created.elapsed() > self.deadline {
            pending.take();
            bail!("Pairing prompt expired");
        }
        let prompt = pending.take().unwrap();
        prompt
            .reply
            .send((accepted, value))
            .map_err(|_| anyhow::anyhow!("Pairing request was cancelled"))?;
        self.state.send_modify(|state| state.prompt = None);
        Ok(())
    }

    pub(crate) fn cancel(&self) {
        self.pending.lock().unwrap().take();
        self.state.send_modify(|state| state.prompt = None);
    }
}
