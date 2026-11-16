//! Typed action plans, validation and the direct/local-model planning policy.
mod direct;
mod llm;

use crate::config::Config;
use anyhow::{bail, Result};
pub use direct::direct;
use serde::{Deserialize, Serialize};

const MAX_ACTIONS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Workspace { id: i32 },
    MoveWindow { id: i32 },
    Focus { app: String },
    Launch { app: String },
    Volume { percent: u8 },
    Mute { muted: bool },
}

impl Action {
    pub fn description(&self) -> String {
        match self {
            Self::Workspace { id } => format!("Switch to workspace {id}"),
            Self::MoveWindow { id } => format!("Move the active window to workspace {id}"),
            Self::Focus { app } => format!("Focus {app}"),
            Self::Launch { app } => format!("Launch {app}"),
            Self::Volume { percent } => format!("Set volume to {percent}%"),
            Self::Mute { muted } => if *muted { "Mute audio" } else { "Unmute audio" }.into(),
        }
    }

    pub fn validate(&self, config: &Config) -> Result<()> {
        match self {
            Self::Workspace { id } | Self::MoveWindow { id } if !(1..=10).contains(id) => {
                bail!("workspace must be between 1 and 10")
            }
            Self::Focus { app }
                if app.is_empty()
                    || app.len() > 64
                    || !app
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ' || c == '.') =>
            {
                bail!("invalid app name")
            }
            Self::Launch { app } if !config.apps.contains_key(app) => {
                bail!("app {app:?} is not in the configured launch allowlist")
            }
            Self::Volume { percent } if *percent > 100 => bail!("volume must be between 0 and 100"),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub actions: Vec<Action>,
    #[serde(default)]
    pub source: String,
}

impl Plan {
    pub fn validate(&self, config: &Config) -> Result<()> {
        if self.actions.is_empty() || self.actions.len() > MAX_ACTIONS {
            bail!("a plan needs between one and four actions");
        }
        for action in &self.actions {
            action.validate(config)?;
        }
        Ok(())
    }
}

pub async fn plan(text: &str, config: &Config) -> Result<Plan> {
    match direct(text, config)? {
        Some(plan) => Ok(plan),
        None => llm::plan(text, config).await,
    }
}
