use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    #[serde(flatten)]
    pub operation: Operation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Status,
    Subscribe,
    ConnectivitySubscribe,
    ConnectivityStatus,
    NotificationsSubscribe,
    NotificationsStatus,
    Lock,
    LockStatus,
    LockSubscribe,
    IdleStatus,
    IdleSubscribe,
    Idle {
        request: crate::services::idle::Request,
    },
    IdleEvent {
        credential: crate::services::idle::Credential,
        event: crate::services::idle::Event,
    },
    SettingsSubscribe,
    SettingsStatus,
    Input {
        request: crate::desktop::input::Request,
    },
    Storage {
        request: crate::services::storage::Request,
    },
    Credentials {
        request: crate::services::credentials::Request,
    },
    SystemKey {
        key: crate::desktop::input::SystemKey,
    },
    PortalsStatus,
    Settings {
        request: crate::services::settings::Request,
    },
    ClipboardSubscribe,
    ClipboardStatus,
    Clipboard {
        request: crate::services::clipboard::Request,
    },
    ClipboardIngest {
        generation: u64,
        text: String,
    },
    CaptureSubscribe,
    CaptureStatus,
    Capture {
        request: crate::services::capture::Request,
    },
    PowerSubscribe,
    PowerStatus,
    Power {
        request: crate::services::power::Request,
    },
    LaunchDesktop {
        desktop_id: String,
    },
    FocusWindow {
        address: String,
    },
    Notifications {
        request: crate::services::notifications::Request,
    },
    Connectivity {
        request: crate::services::connectivity::Request,
    },
    Command {
        text: String,
        #[serde(default = "yes")]
        dry_run: bool,
    },
    Confirm {
        token: u64,
    },
    Cancel,
    Toggle,
    Panel {
        panel: DesktopPanel,
        open: bool,
    },
    Voice {
        mode: VoiceMode,
    },
}

/// On-demand shell surfaces coordinated by the daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopPanel {
    Audio,
    Power,
    Tray,
    Settings,
    Clipboard,
    Capture,
    Storage,
    Launcher,
    Switcher,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum VoiceMode {
    #[default]
    Off,
    Wake,
    Dictate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub phase: String,
    pub message: String,
    pub transcript: String,
    pub actions: Vec<String>,
    pub receipts: Vec<String>,
    pub confirmation: Option<u64>,
    pub panel_open: bool,
    pub desktop_panel: Option<DesktopPanel>,
    pub voice: VoiceMode,
    pub workspace: Option<i32>,
    pub active_app: String,
    #[serde(default)]
    pub feedback: Option<Feedback>,
    #[serde(default)]
    pub credentials: crate::services::credentials::State,
    #[serde(default)]
    pub storage: crate::services::storage::State,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").into(),
            phase: "ready".into(),
            message: "Your desktop, within reach.".into(),
            transcript: String::new(),
            actions: vec![],
            receipts: vec![],
            confirmation: None,
            panel_open: false,
            desktop_panel: None,
            voice: VoiceMode::Off,
            workspace: None,
            active_app: String::new(),
            feedback: None,
            credentials: crate::services::credentials::State::default(),
            storage: crate::services::storage::State::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub sequence: u64,
    pub issued_at: u64,
    pub title: String,
    pub detail: String,
    pub icon: String,
    pub percent: Option<u8>,
}
