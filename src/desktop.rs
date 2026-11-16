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
                if uses_lua(conn).await? {
                    conn.dispatch(&format!("hl.dsp.focus({{ workspace = {id} }})"))
                        .await?;
                } else {
                    conn.switch_workspace(&WorkspaceRef::Id(*id)).await?;
                }
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
                if uses_lua(conn).await? {
                    let selector = lua_window(&window.address)?;
                    conn.dispatch(&format!(
                        "hl.dsp.window.move({{ workspace = {id}, window = {selector}, follow = true }})"
                    ))
                    .await?;
                } else {
                    conn.dispatch(&format!(
                        "movetoworkspace {id},{}",
                        WindowSelector::Address(window.address.clone()).encode()
                    ))
                    .await?;
                }
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
                if uses_lua(conn).await? {
                    let selector = lua_window(&target.address)?;
                    conn.dispatch(&format!("hl.dsp.focus({{ window = {selector} }})"))
                        .await?;
                } else {
                    conn.focus_window(&WindowSelector::Address(target.address.clone()))
                        .await?;
                }
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
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos();
                let unit = format!(
                    "hyprlume-app-{}-{}-{stamp}",
                    app.replace(' ', "-"),
                    std::process::id()
                );
                // The user manager owns the app independently of this daemon.
                // A daemon restart must not terminate an app or inherit its
                // service sandbox. Type=exec confirms that exec itself worked.
                let mut args = vec![
                    "--user".to_owned(),
                    "--collect".into(),
                    "--quiet".into(),
                    "--service-type=exec".into(),
                    "--expand-environment=no".into(),
                    format!("--unit={unit}"),
                ];
                for name in [
                    "DISPLAY",
                    "WAYLAND_DISPLAY",
                    "XDG_RUNTIME_DIR",
                    "XDG_CURRENT_DESKTOP",
                    "XDG_SESSION_TYPE",
                    "DBUS_SESSION_BUS_ADDRESS",
                    "XAUTHORITY",
                    "LANG",
                    "LC_ALL",
                    "PATH",
                    "QT_QPA_PLATFORM",
                    "QT_QPA_PLATFORMTHEME",
                    "GDK_BACKEND",
                    "SSH_AUTH_SOCK",
                ] {
                    if std::env::var_os(name).is_some() {
                        args.push(format!("--setenv={name}"));
                    }
                }
                args.push("--".into());
                args.extend(argv.iter().cloned());
                let args: Vec<_> = args.iter().map(String::as_str).collect();
                run("systemd-run", &args)
                    .await
                    .with_context(|| format!("launch {app} through the systemd user manager"))?;
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

// The pinned HyprPilot transport predates Lua dispatchers. Detect the same
// configProvider as Quickshell, while retaining its typed legacy operations.
// Read at action time so a config reload cannot leave a stale mode cached.
async fn uses_lua(conn: &Connection) -> Result<bool> {
    let response = conn.send_raw("j/status").await?;
    if response.trim() == "unknown request" {
        return Ok(false);
    }
    #[derive(serde::Deserialize)]
    struct Status {
        #[serde(rename = "configProvider")]
        provider: String,
    }
    let status: Status =
        serde_json::from_str(&response).context("Hyprland returned invalid config status")?;
    match status.provider.as_str() {
        "lua" => Ok(true),
        "hyprlang" => Ok(false),
        other => bail!("Unsupported Hyprland config provider: {other}"),
    }
}

fn lua_window(address: &str) -> Result<String> {
    let hex = address.strip_prefix("0x").unwrap_or("");
    if hex.is_empty() || hex.len() > 16 || !hex.bytes().all(|ch| ch.is_ascii_hexdigit()) {
        bail!("Hyprland returned an invalid window address");
    }
    Ok(format!("\"address:{address}\""))
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
