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
    Voice {
        mode: VoiceMode,
    },
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
    pub voice: VoiceMode,
    pub workspace: Option<i32>,
    pub active_app: String,
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
            voice: VoiceMode::Off,
            workspace: None,
            active_app: String::new(),
        }
    }
}
