use crate::{config::Config, planner::Action};
use anyhow::{bail, Context, Result};
use hyprpilot_core::{Connection, WindowSelector, WorkspaceRef};
use std::{process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Clone)]
pub struct Desktop {
    pub connection: Option<Connection>,
    pub demo: bool,
}

impl Desktop {
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
            Action::Workspace { id } => {
                let conn = self
                    .connection
                    .as_ref()
                    .context("No Hyprland session; start hyprlume inside your own session")?;
                conn.switch_workspace(&WorkspaceRef::Id(*id)).await?;
                if conn.active_workspace().await?.id != *id {
                    bail!("Hyprland did not confirm the requested workspace");
                }
                Ok(format!("Workspace {id} confirmed"))
            }
            Action::MoveWindow { id } => {
                let conn = self.connection.as_ref().context("No Hyprland session")?;
                let target = target.context("No window was bound to this move")?;
                let window = conn
                    .clients()
                    .await?
                    .into_iter()
                    .find(|w| w.address == target)
                    .context("The target window is no longer open")?;
                // Bind to this address; an overlay or a focus event must not change the target.
                conn.dispatch(&format!(
                    "movetoworkspace {id},{}",
                    WindowSelector::Address(window.address.clone()).encode()
                ))
                .await?;
                let moved = conn
                    .clients()
                    .await?
                    .into_iter()
                    .any(|w| w.address == window.address && w.workspace.id == *id);
                if !moved {
                    bail!("Hyprland did not confirm the window move");
                }
                Ok(format!("{} moved to workspace {id}", window.class))
            }
            Action::Focus { app } => {
                let conn = self.connection.as_ref().context("No Hyprland session")?;
                let matches: Vec<_> = conn
                    .clients()
                    .await?
                    .into_iter()
                    .filter(|w| w.class.eq_ignore_ascii_case(app))
                    .collect();
                if matches.len() != 1 {
                    bail!(
                        "Expected one {app} window, found {}; focus it manually to disambiguate",
                        matches.len()
                    );
                }
                let target = &matches[0];
                conn.focus_window(&WindowSelector::Address(target.address.clone()))
                    .await?;
                if !conn
                    .active_window()
                    .await?
                    .is_some_and(|w| w.address == target.address)
                {
                    bail!("Hyprland did not confirm the focused window");
                }
                Ok(format!("{} focused", target.class))
            }
            Action::Launch { app } => {
                let argv = &config.apps[app];
                let mut child = Command::new(&argv[0])
                    .args(&argv[1..])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .with_context(|| format!("launch {app}"))?;
                tokio::spawn(async move {
                    let _ = child.wait().await;
                });
                Ok(format!("{app} launch requested · process started"))
            }
            Action::Volume { percent } => {
                run(
                    "wpctl",
                    &["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{percent}%")],
                )
                .await?;
                let actual = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]).await?;
                let volume: f32 = actual
                    .split_whitespace()
                    .nth(1)
                    .context("wpctl returned no volume")?
                    .parse()?;
                if (volume * 100.0 - *percent as f32).abs() > 1.1 {
                    bail!("audio server did not confirm the requested volume");
                }
                Ok(format!("Volume {percent}% confirmed"))
            }
            Action::Mute { muted } => {
                run(
                    "wpctl",
                    &[
                        "set-mute",
                        "@DEFAULT_AUDIO_SINK@",
                        if *muted { "1" } else { "0" },
                    ],
                )
                .await?;
                let actual = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]).await?;
                if actual.contains("[MUTED]") != *muted {
                    bail!("audio server did not confirm mute state");
                }
                Ok(if *muted {
                    "Audio muted"
                } else {
                    "Audio unmuted"
                }
                .into())
            }
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
