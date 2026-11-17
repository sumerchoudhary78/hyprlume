//! Opt-in, bounded, volatile text history. Clipboard payloads never enter agent state.
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    sync::{watch, Mutex as AsyncMutex, Semaphore},
    task::JoinHandle,
    time::{timeout, Duration},
};
use zeroize::Zeroizing;

pub const MAX_TEXT: usize = 3072;
const MAX_ITEMS: usize = 24;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,
    pub preview: String,
    pub bytes: usize,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub enabled: bool,
    pub available: bool,
    pub items: Vec<Entry>,
    pub error: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Enable { enabled: bool },
    Copy { id: u64 },
    Remove { id: u64 },
    Clear,
}

struct Stored {
    id: u64,
    text: Zeroizing<String>,
}
pub struct Controller {
    state: watch::Sender<State>,
    entries: Mutex<VecDeque<Stored>>,
    sequence: AtomicU64,
    generation: AtomicU64,
    watcher: AsyncMutex<Option<JoinHandle<()>>>,
    admission: Semaphore,
    socket: PathBuf,
    demo: bool,
}
impl Controller {
    pub fn new(socket: PathBuf, demo: bool) -> Arc<Self> {
        let available = super::tools::available("wl-paste") && super::tools::available("wl-copy");
        let (state, _) = watch::channel(State {
            available,
            ..State::default()
        });
        Arc::new(Self {
            state,
            entries: Mutex::default(),
            sequence: AtomicU64::new(1),
            generation: AtomicU64::new(0),
            watcher: AsyncMutex::default(),
            admission: Semaphore::new(1),
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
    fn publish(&self) {
        let entries = self.entries.lock().unwrap();
        self.state.send_modify(|state| {
            state.items = entries
                .iter()
                .map(|entry| Entry {
                    id: entry.id,
                    bytes: entry.text.len(),
                    preview: entry
                        .text
                        .chars()
                        .take(160)
                        .map(|c| if c.is_control() { ' ' } else { c })
                        .collect(),
                })
                .collect()
        });
    }
    pub fn ingest(&self, generation: u64, text: String) -> Result<()> {
        let text = Zeroizing::new(text);
        ensure!(
            !text.is_empty() && text.len() <= MAX_TEXT && !text.contains('\0'),
            "Clipboard text is empty, binary or too large"
        );
        let mut entries = self.entries.lock().unwrap();
        // Check while holding the store lock so a pause/clear cannot race a late delivery.
        ensure!(
            self.state.borrow().enabled && generation == self.generation.load(Ordering::SeqCst),
            "Clipboard history is paused"
        );
        entries.retain(|entry| *entry.text != *text);
        entries.push_front(Stored {
            id: self.sequence.fetch_add(1, Ordering::Relaxed),
            text,
        });
        entries.truncate(MAX_ITEMS);
        drop(entries);
        self.publish();
        Ok(())
    }
    pub async fn handle(self: &Arc<Self>, request: Request) -> Result<State> {
        let _permit = self
            .admission
            .try_acquire()
            .context("Another clipboard operation is in progress")?;
        match request {
            Request::Enable { enabled } => {
                self.stop_watcher().await;
                if enabled {
                    ensure!(
                        self.demo || self.state().available,
                        "Install wl-clipboard to enable history"
                    );
                    ensure!(
                        self.demo || std::env::var_os("WAYLAND_DISPLAY").is_some(),
                        "Clipboard history requires a Wayland session"
                    );
                    let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
                    if !self.demo {
                        self.state.send_modify(|s| {
                            s.enabled = true;
                            s.error.clear();
                        });
                        let child = Command::new("wl-paste")
                            .args(["--type", "text", "--watch"])
                            .arg(std::env::current_exe()?)
                            .arg("--socket")
                            .arg(&self.socket)
                            .args([
                                "clipboard",
                                "ingest",
                                "--generation",
                                &generation.to_string(),
                            ])
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .kill_on_drop(true)
                            .spawn();
                        let mut child = match child {
                            Ok(child) => child,
                            Err(error) => {
                                self.state.send_modify(|s| s.enabled = false);
                                return Err(error).context("Start Wayland clipboard watcher");
                            }
                        };
                        let weak = Arc::downgrade(self);
                        *self.watcher.lock().await = Some(tokio::spawn(async move {
                            let _ = child.wait().await;
                            if let Some(controller) = weak.upgrade() {
                                if controller.generation.load(Ordering::SeqCst) == generation {
                                    controller.generation.fetch_add(1, Ordering::SeqCst);
                                    controller.state.send_modify(|s| {
                                        s.enabled = false;
                                        s.error =
                                            "Clipboard watcher stopped; enable it again to retry"
                                                .into();
                                    });
                                }
                            }
                        }));
                    } else {
                        self.state.send_modify(|s| {
                            s.enabled = true;
                            s.error.clear();
                        });
                    }
                }
            }
            Request::Clear => {
                self.entries.lock().unwrap().clear();
                self.publish();
            }
            Request::Remove { id } => {
                self.entries.lock().unwrap().retain(|entry| entry.id != id);
                self.publish();
            }
            Request::Copy { id } => {
                let text = self
                    .entries
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| entry.text.clone())
                    .context("This clipboard entry is no longer available")?;
                if !self.demo {
                    let mut child = Command::new("wl-copy")
                        .args(["--type", "text/plain;charset=utf-8"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .kill_on_drop(true)
                        .spawn()
                        .context("Start wl-copy")?;
                    timeout(Duration::from_secs(2), async {
                        child
                            .stdin
                            .take()
                            .context("Clipboard input unavailable")?
                            .write_all(text.as_bytes())
                            .await?;
                        ensure!(
                            child.wait().await?.success(),
                            "Wayland clipboard copy failed"
                        );
                        Ok::<_, anyhow::Error>(())
                    })
                    .await
                    .context("Clipboard copy timed out")??;
                }
            }
        }
        Ok(self.state())
    }
    async fn stop_watcher(&self) {
        let _entries = self.entries.lock().unwrap();
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.state.send_modify(|s| s.enabled = false);
        drop(_entries);
        if let Some(task) = self.watcher.lock().await.take() {
            task.abort();
            let _ = task.await;
        }
    }
    pub async fn shutdown(&self) {
        let _permit = self.admission.acquire().await;
        self.stop_watcher().await;
        self.entries.lock().unwrap().clear();
        self.publish();
    }
}

/// Used only by wl-paste's short-lived delivery process. Sensitive offers are
/// discarded before stdin is read; the protocol cannot reliably identify every password.
pub fn read_offer() -> Result<Option<String>> {
    if matches!(
        std::env::var("CLIPBOARD_STATE").as_deref(),
        Ok("sensitive" | "clear" | "nil")
    ) {
        return Ok(None);
    }
    use std::io::Read;
    let mut bytes = Zeroizing::new(Vec::new());
    std::io::stdin()
        .take((MAX_TEXT + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > MAX_TEXT {
        bail!("Clipboard offer exceeds the text history limit");
    }
    Ok(Some(
        String::from_utf8(bytes.to_vec()).context("Clipboard offer is not UTF-8 text")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn history_is_opt_in_bounded_deduplicated_and_stale_deliveries_are_revoked() {
        let c = Controller::new(PathBuf::from("/unused"), true);
        assert!(c.ingest(0, "secret".into()).is_err());
        c.handle(Request::Enable { enabled: true }).await.unwrap();
        let generation = c.generation.load(Ordering::SeqCst);
        for i in 0..30 {
            c.ingest(generation, format!("item {i}")).unwrap();
        }
        assert_eq!(c.state().items.len(), 24);
        c.ingest(generation, "item 29".into()).unwrap();
        assert_eq!(c.state().items.len(), 24);
        assert!(!serde_json::to_string(&c.state())
            .unwrap()
            .contains("generation"));
        let id = c.state().items[0].id;
        c.handle(Request::Remove { id }).await.unwrap();
        assert!(c.handle(Request::Copy { id }).await.is_err());
        c.handle(Request::Enable { enabled: false }).await.unwrap();
        assert!(c.ingest(generation, "late offer".into()).is_err());
        c.handle(Request::Enable { enabled: true }).await.unwrap();
        assert!(c.ingest(generation, "old watcher".into()).is_err());
        c.shutdown().await;
        assert!(c.state().items.is_empty());
    }
}
