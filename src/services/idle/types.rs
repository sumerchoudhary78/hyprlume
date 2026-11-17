use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub enabled: bool,
    pub available: bool,
    pub presentation: bool,
    pub recording_inhibits: bool,
    pub dimmed: bool,
    pub display_off: bool,
    pub phase: String,
    pub message: String,
    pub error: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Dim,
    Lock,
    Off,
    Suspend,
    Resume,
    BeforeSleep,
    AfterResume,
    ForceLock,
}

impl Event {
    pub(super) fn bit(self) -> u8 {
        match self {
            Self::Dim => 1,
            Self::Lock => 2,
            Self::Off => 4,
            Self::Suspend => 8,
            _ => 0,
        }
    }
}

/// Provider callbacks carry a per-launch capability, never exposed in status or logs.
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Credential(pub(super) String);
impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Presentation { enabled: bool },
}
