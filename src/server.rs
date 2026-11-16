use crate::{
    config::Config,
    desktop::Desktop,
    planner::{self, Plan},
    protocol::{Operation, Request, State, VoiceMode},
};
use anyhow::{bail, Context, Result};
use fs2::FileExt;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
#[cfg(feature = "voice")]
use std::sync::atomic::AtomicBool;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    net::{UnixListener, UnixStream},
    sync::{watch, Mutex, Semaphore},
    time::timeout,
};
use tokio_util::codec::{Framed, LinesCodec};

pub const MAX_FRAME: usize = 8192;

pub fn socket_path() -> Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR is required; run inside your user session")?;
    Ok(PathBuf::from(runtime).join("hyprlume/control.sock"))
}

struct SocketGuard {
    path: PathBuf,
    _lock: File,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn bind(path: &Path) -> Result<(UnixListener, SocketGuard)> {
    let parent = path.parent().context("socket path needs a parent")?;
    match fs::DirBuilder::new().mode(0o700).create(parent) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o077 != 0
    {
        bail!("socket directory must be a private directory (0700)");
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(parent.join("daemon.lock"))?;
    lock.try_lock_exclusive()
        .context("another hyprlume daemon is already running")?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.file_type().is_socket() {
            bail!("refusing to replace a non-socket at {}", path.display());
        }
        fs::remove_file(path)?;
    }
    let listener = UnixListener::bind(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok((
        listener,
        SocketGuard {
            path: path.to_owned(),
            _lock: lock,
        },
    ))
}

struct Pending {
    token: u64,
    plan: Plan,
    created: Instant,
    active_window: Option<String>,
}

pub struct App {
    pub config: Config,
    pub desktop: Desktop,
    state: watch::Sender<State>,
    pending: Mutex<Option<Pending>>,
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

    pub async fn handle(self: &Arc<Self>, op: Operation) -> Result<Value> {
        match op {
            Operation::Status | Operation::Subscribe => Ok(json!(self.state())),
            Operation::Toggle => {
                self.update(|s| s.panel_open = !s.panel_open);
                Ok(json!(self.state()))
            }
            Operation::Cancel => {
                if self.busy.available_permits() == 0 {
                    bail!("a command is running; dismiss its preview after planning completes");
                }
                self.pending.lock().await.take();
                self.update(|s| {
                    s.confirmation = None;
                    s.actions.clear();
                    s.phase = "ready".into();
                    s.message = "Preview dismissed".into();
                });
                Ok(json!(self.state()))
            }
            Operation::Voice { mode } => {
                self.set_voice(mode).await?;
                Ok(json!(self.state()))
            }
            Operation::Command { text, dry_run } => self.submit(text, dry_run).await,
            Operation::Confirm { token } => {
                let permit = self
                    .busy
                    .clone()
                    .try_acquire_owned()
                    .context("another command is running")?;
                let mut pending = self.pending.lock().await;
                let plan = pending
                    .as_ref()
                    .context("no action is waiting for confirmation")?;
                if plan.token != token {
                    bail!("confirmation token does not match");
                }
                let plan = pending.take().unwrap();
                drop(pending);
                self.update(|s| s.confirmation = None);
                if plan.created.elapsed() > Duration::from_secs(30) {
                    self.fail("Preview expired; request the action again");
                    bail!("preview expired after 30 seconds");
                }
                if let Some(address) = &plan.active_window {
                    let conn = self
                        .desktop
                        .connection
                        .as_ref()
                        .context("Hyprland disconnected")?;
                    if !conn
                        .active_window()
                        .await?
                        .is_some_and(|w| w.address == *address)
                    {
                        self.fail("Window focus changed; request the move again");
                        bail!("window focus changed since preview");
                    }
                }
                let app = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = app.execute(&plan.plan, plan.active_window.as_deref()).await {
                        app.fail(&e.to_string());
                    }
                    drop(permit);
                });
                Ok(json!({"accepted":true}))
            }
        }
    }

    async fn submit(self: &Arc<Self>, text: String, dry_run: bool) -> Result<Value> {
        let permit = self
            .busy
            .clone()
            .try_acquire_owned()
            .context("another command is running")?;
        if self.pending.lock().await.is_some() {
            bail!("confirm or dismiss the current preview first");
        }
        // Validate the text before publishing it or sending it to a model.
        let direct = planner::direct(&text, &self.config)?;
        self.update(|s| {
            s.phase = "planning".into();
            s.message = "Preparing your action".into();
            s.transcript = text.clone();
            s.receipts.clear();
            s.actions.clear();
            s.confirmation = None;
        });
        let app = self.clone();
        tokio::spawn(async move {
            let result = async {
                let plan = match direct {
                    Some(plan) => plan,
                    None => planner::plan(&text, &app.config).await?,
                };
                let active_window = if plan
                    .actions
                    .iter()
                    .any(|a| matches!(a, planner::Action::MoveWindow { .. }))
                {
                    if let Some(conn) = &app.desktop.connection {
                        conn.active_window().await?.map(|w| w.address)
                    } else {
                        None
                    }
                } else {
                    None
                };
                app.update(|s| s.actions = plan.actions.iter().map(|a| a.description()).collect());
                if dry_run || plan.source == "local_llm" {
                    let token = app.sequence.fetch_add(1, Ordering::Relaxed);
                    *app.pending.lock().await = Some(Pending {
                        token,
                        plan,
                        created: Instant::now(),
                        active_window,
                    });
                    app.update(|s| {
                        s.phase = "review".into();
                        s.message = "Review the action before running it".into();
                        s.confirmation = Some(token);
                        s.panel_open = true;
                    });
                } else {
                    app.execute(&plan, active_window.as_deref()).await?;
                }
                Ok::<_, anyhow::Error>(())
            }
            .await;
            if let Err(e) = result {
                app.fail(&e.to_string());
            }
            drop(permit);
        });
        Ok(json!({"accepted":true}))
    }

    fn fail(&self, message: &str) {
        self.update(|s| {
            s.phase = "error".into();
            s.message = message.chars().take(512).collect();
            s.confirmation = None;
            s.panel_open = true;
        });
    }

    async fn execute(&self, plan: &Plan, target: Option<&str>) -> Result<()> {
        self.update(|s| {
            s.phase = "executing".into();
            s.message = "Applying your action".into();
        });
        for action in &plan.actions {
            let receipt = self.desktop.execute(action, &self.config, target).await?;
            self.update(|s| s.receipts.push(receipt));
        }
        self.update(|s| {
            s.phase = "done".into();
            s.message = if self.desktop.demo {
                "Demo complete · no desktop changes"
            } else {
                "Action completed"
            }
            .into();
        });
        #[cfg(feature = "voice")]
        if self.config.voice.speak_results && self.state().voice == VoiceMode::Wake {
            if let Err(e) = self.feedback().await {
                tracing::warn!(error = %e, "spoken feedback unavailable");
                self.update(|s| {
                    s.phase = "done".into();
                    s.message = "Action completed · spoken feedback unavailable".into();
                });
            }
        }
        Ok(())
    }

    #[cfg(feature = "voice")]
    async fn feedback(&self) -> Result<()> {
        struct Pause(Arc<AtomicBool>);
        impl Drop for Pause {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Relaxed);
            }
        }
        self.audio_paused.store(true, Ordering::Relaxed);
        let _pause = Pause(self.audio_paused.clone());
        let output = socket_path()?.parent().unwrap().join("feedback.wav");
        let cache = self.speaker.clone();
        let config = self.config.voice.clone();
        let text: String = self.state().receipts.join(". ").chars().take(400).collect();
        let path = output.clone();
        self.update(|s| s.phase = "speaking".into());
        tokio::task::spawn_blocking(move || {
            let mut speaker = cache
                .lock()
                .map_err(|_| anyhow::anyhow!("TTS cache lock failed"))?;
            if speaker.is_none() {
                *speaker = Some(crate::voice::Speaker::new(&config)?);
            }
            speaker.as_ref().unwrap().synthesize(&text, &path)
        })
        .await??;
        let playback = tokio::process::Command::new("pw-play")
            .arg(&output)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .status();
        let played = timeout(Duration::from_secs(20), playback).await;
        let _ = fs::remove_file(output);
        if !played.context("speech playback timed out")??.success() {
            bail!("PipeWire playback failed");
        }
        // Drain microphone echo/backlog before arming the keyword spotter again.
        tokio::time::sleep(Duration::from_millis(300)).await;
        self.update(|s| s.phase = "done".into());
        Ok(())
    }

    async fn set_voice(self: &Arc<Self>, mode: VoiceMode) -> Result<()> {
        #[cfg(not(feature = "voice"))]
        {
            let _ = mode;
            bail!("this build has no voice feature");
        }
        #[cfg(feature = "voice")]
        {
            let mut voice = self.voice.lock().await;
            if let Some(task) = voice.take() {
                task.abort();
                let _ = task.await;
            }
            self.update(|s| {
                s.voice = VoiceMode::Off;
                if s.confirmation.is_none()
                    && !matches!(s.phase.as_str(), "planning" | "executing" | "speaking")
                {
                    s.phase = "ready".into();
                    s.message = "Microphone off".into();
                }
            });
            if mode == VoiceMode::Off {
                return Ok(());
            }
            if self.desktop.demo {
                bail!("microphone capture is unavailable in demo mode");
            }
            crate::voice::check_models(&self.config.voice, mode)?;
            self.update(|s| {
                s.voice = mode;
                if s.confirmation.is_none()
                    && !matches!(s.phase.as_str(), "planning" | "executing" | "speaking")
                {
                    s.message = "Starting local microphone capture".into();
                }
            });
            let app = self.clone();
            *voice = Some(tokio::spawn(async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel(8);
                let config = app.config.voice.clone();
                let worker = crate::voice::listen(config, mode, tx, app.audio_paused.clone());
                tokio::pin!(worker);
                let result = loop {
                    tokio::select! {
                        result = &mut worker => break result,
                        event = rx.recv() => if let Some(event) = event {
                            // A second wake must not overwrite a pending preview or
                            // a running action. The next request needs a new wake.
                            if app.busy.available_permits() == 0 || app.state().confirmation.is_some() { continue; }
                            match event {
                                crate::voice::VoiceEvent::Ready => app.update(|s| { s.voice = mode; s.phase = "ready".into(); s.message = if mode == VoiceMode::Wake { "Listening for ‘Hey Pilot’" } else { "Dictation · text stays in this panel" }.into(); }),
                                crate::voice::VoiceEvent::Awake => app.update(|s| { s.phase = "listening".into(); s.panel_open = true; s.transcript.clear(); s.message = "Listening to your command".into(); }),
                                crate::voice::VoiceEvent::Partial(text) => app.update(|s| s.transcript = text),
                                crate::voice::VoiceEvent::Final(text) => {
                                    if mode == VoiceMode::Dictate {
                                        app.update(|s| { s.phase = "ready".into(); s.transcript = text; s.message = "Dictation ready · no desktop action".into(); s.panel_open = true; });
                                    } else {
                                        if let Err(e) = app.handle_voice_command(text).await { app.fail(&e.to_string()); }
                                    }
                                }
                            }
                        } else { break Ok(()); }
                    }
                };
                if let Err(e) = result {
                    let message = format!("Voice stopped: {e}");
                    if app.busy.available_permits() == 0 || app.state().confirmation.is_some() {
                        app.update(|s| s.message = message.chars().take(512).collect());
                    } else {
                        app.fail(&message);
                    }
                }
                app.update(|s| s.voice = VoiceMode::Off);
            }));
            Ok(())
        }
    }

    #[cfg(feature = "voice")]
    async fn handle_voice_command(self: &Arc<Self>, text: String) -> Result<()> {
        // Never interpret the output of dictation as a desktop action.
        self.submit(text, !self.config.voice.execute_commands)
            .await?;
        Ok(())
    }
}

async fn session(stream: UnixStream, app: Arc<App>) -> Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(MAX_FRAME));
    let mut state = app.state.subscribe();
    let mut subscribed = false;
    loop {
        let output = tokio::select! {
            input = framed.next() => {
                let Some(input) = input else { return Ok(()); };
                let input = input?;
                match serde_json::from_str::<Request>(&input) {
                    Ok(request) => {
                        if matches!(request.operation, Operation::Subscribe) { subscribed = true; state.borrow_and_update(); }
                        match app.handle(request.operation).await {
                            Ok(result) => json!({"id":request.id,"result":result}),
                            Err(e) => json!({"id":request.id,"error":e.to_string()}),
                        }
                    }
                    Err(_) => json!({"id":null,"error":"invalid request"}),
                }
            }
            change = state.changed(), if subscribed => {
                change?;
                let snapshot = state.borrow_and_update().clone();
                json!({"event":"state","state":snapshot})
            }
        };
        timeout(Duration::from_secs(2), framed.send(output.to_string())).await??;
    }
}

async fn desktop_events(app: Arc<App>) {
    let Some(conn) = app.desktop.connection.clone() else {
        return;
    };
    loop {
        let result = async {
            let mut stream = hyprpilot_core::EventStream::connect(&conn).await?;
            if let Ok(ws) = conn.active_workspace().await {
                app.update(|s| s.workspace = Some(ws.id));
            }
            if let Ok(Some(window)) = conn.active_window().await {
                app.update(|s| s.active_app = window.class);
            }
            while let Some(event) = stream.next().await? {
                let event = hyprpilot_core::Event::from_raw(event);
                match event.kind.as_str() {
                    "workspacev2" => {
                        if let Some(id) = event.field("id").and_then(|v| v.parse().ok()) {
                            app.update(|s| s.workspace = Some(id));
                        }
                    }
                    "activewindow" => {
                        if let Some(class) = event.field("class") {
                            app.update(|s| s.active_app = class.chars().take(64).collect());
                        }
                    }
                    _ => {}
                }
            }
            Ok::<_, anyhow::Error>(())
        }
        .await;
        tracing::debug!(?result, "Hyprland event stream disconnected");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

pub async fn serve(path: PathBuf, config: Config, demo: bool) -> Result<()> {
    config.validate()?;
    let (listener, _guard) = bind(&path)?;
    let app = App::new(config, demo);
    let events = tokio::spawn(desktop_events(app.clone()));
    let connections = Arc::new(Semaphore::new(16));
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tracing::info!(socket = %path.display(), "hyprlume ready; microphone off");
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = terminate.recv() => break,
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let Ok(permit) = connections.clone().try_acquire_owned() else { continue; };
                let app = app.clone();
                tokio::spawn(async move { if let Err(e) = session(stream, app).await { tracing::debug!(error = %e, "client disconnected"); } drop(permit); });
            }
        }
    }
    app.set_voice(VoiceMode::Off).await.ok();
    events.abort();
    Ok(())
}

pub async fn call(path: &Path, operation: Operation) -> Result<Value> {
    let stream = timeout(Duration::from_secs(2), UnixStream::connect(path))
        .await?
        .with_context(|| format!("connect {}; start ‘hyprlume daemon’ first", path.display()))?;
    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(MAX_FRAME));
    framed
        .send(serde_json::to_string(&Request { id: 1, operation })?)
        .await?;
    let response = timeout(Duration::from_secs(3), framed.next())
        .await?
        .context("daemon closed connection")??;
    let response: Value = serde_json::from_str(&response)?;
    if let Some(error) = response.get("error") {
        bail!("{}", error.as_str().unwrap_or("request failed"));
    }
    Ok(response["result"].clone())
}
