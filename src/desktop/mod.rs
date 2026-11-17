//! Concrete desktop adapters. Validation and demo handling happen once here.
mod applications;
mod audio;
mod display;
mod hyprland;
pub(crate) use display::Display;

use crate::{config::Config, planner::Action};
use anyhow::{bail, Context, Result};
use hyprpilot_core::Connection;
use std::{process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Clone)]
pub struct Desktop {
    pub connection: Option<Connection>,
    pub demo: bool,
}

impl Desktop {
    pub(crate) async fn launch_desktop(&self, id: &str) -> Result<String> {
        let argv = applications::desktop_argv(id)?;
        if self.demo {
            return Ok(format!("Demo only · launch desktop application {id}"));
        }
        applications::launch("desktop", &argv).await
    }
    pub fn new(demo: bool) -> Self {
        Self {
            connection: if demo { None } else { Connection::new().ok() },
            demo,
        }
    }

    pub async fn execute(
        &self,
        action: &Action,
        config: &Config,
        target: Option<&str>,
    ) -> Result<String> {
        action.validate(config)?;
        if self.demo {
            return Ok(format!("Demo only · {}", action.description()));
        }
        match action {
            Action::Workspace { id } => self.switch_workspace(*id).await,
            Action::MoveWindow { id } => self.move_window(*id, target).await,
            Action::Focus { app } => self.focus_window(app).await,
            Action::Launch { app } => applications::launch(app, &config.apps[app]).await,
            Action::Volume { percent } => audio::set_volume(*percent).await,
            Action::Mute { muted } => audio::set_mute(*muted).await,
        }
    }
}

async fn run(program: &str, args: &[&str]) -> Result<String> {
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("start {program}"))?;
    let output = tokio::time::timeout(Duration::from_secs(3), child.wait_with_output())
        .await
        .context("desktop action timed out")??;
    if !output.status.success() {
        bail!(
            "{program}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}
