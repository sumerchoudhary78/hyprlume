use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    PowerSaver,
    Balanced,
    Performance,
}
impl Profile {
    #[cfg(target_os = "linux")]
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::PowerSaver => "power-saver",
            Self::Balanced => "balanced",
            Self::Performance => "performance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    Suspend,
    Logout,
    Reboot,
    PowerOff,
}
impl SessionAction {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Suspend => "Suspend",
            Self::Logout => "Log out",
            Self::Reboot => "Restart",
            Self::PowerOff => "Shut down",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Refresh,
    Brightness { device: String, percent: u8 },
    Profile { profile: Profile },
    Prepare { kind: SessionAction },
    Confirm { token: u64 },
    Cancel { token: u64 },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Battery {
    pub available: bool,
    pub present: bool,
    pub percentage: Option<f64>,
    pub state: String,
    pub seconds_remaining: Option<u64>,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backlight {
    pub id: String,
    pub name: String,
    pub percent: u8,
    pub value: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profiles {
    pub available: bool,
    pub active: Option<Profile>,
    pub choices: Vec<Profile>,
    pub degraded: String,
    pub error: String,
    #[serde(skip)]
    pub(crate) binding: Option<Binding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub available: bool,
    pub id: String,
    pub suspend: String,
    pub logout: String,
    pub reboot: String,
    pub power_off: String,
    pub error: String,
    #[serde(skip)]
    pub(crate) target: Option<SessionTarget>,
}
impl Session {
    pub fn capability(&self, action: SessionAction) -> &str {
        match action {
            SessionAction::Suspend => &self.suspend,
            SessionAction::Logout => &self.logout,
            SessionAction::Reboot => &self.reboot,
            SessionAction::PowerOff => &self.power_off,
        }
    }
    pub(crate) fn validate(&self, action: SessionAction) -> Result<()> {
        ensure!(
            self.available && self.target.is_some(),
            "No verified graphical session is available"
        );
        ensure!(
            matches!(self.capability(action), "yes" | "challenge"),
            "{} is unavailable or inhibited",
            action.label()
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Hardware {
    pub battery: Battery,
    pub backlights: Vec<Backlight>,
    pub profiles: Profiles,
    pub session: Session,
}
impl Hardware {
    pub(crate) fn validate(&self, request: &Request) -> Result<()> {
        match request {
            Request::Brightness { device, percent } => {
                ensure!(
                    (5..=100).contains(percent),
                    "Brightness must be between 5 and 100%"
                );
                ensure!(
                    self.session.available && self.session.target.is_some(),
                    "Brightness needs a verified graphical session"
                );
                ensure!(
                    device.len() <= 256 && self.backlights.iter().any(|light| &light.id == device),
                    "Backlight changed or is no longer available"
                );
            }
            Request::Profile { profile } => {
                ensure!(
                    self.profiles.available
                        && self.profiles.binding.is_some()
                        && self.profiles.choices.contains(profile),
                    "Power profile is unavailable"
                );
            }
            Request::Prepare { kind } => self.session.validate(*kind)?,
            _ => (),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    pub token: u64,
    pub kind: SessionAction,
    pub session_id: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    #[serde(flatten)]
    pub hardware: Hardware,
    pub busy: bool,
    pub message: String,
    pub error: String,
    pub prompt: Option<Prompt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binding {
    pub owner: String,
    pub path: String,
    pub interface: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionTarget {
    pub binding: Binding,
    pub id: String,
    pub uid: u32,
    pub timestamp: u64,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) enum Mutation {
    Brightness {
        device: String,
        percent: u8,
    },
    Profile(Profile),
    Session {
        kind: SessionAction,
        target: SessionTarget,
    },
}
