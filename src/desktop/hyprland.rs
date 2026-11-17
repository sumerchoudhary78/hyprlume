//! HyprPilot-backed workspace/window control, with observed postconditions.
use super::Desktop;
use anyhow::{bail, Context, Result};
use hyprpilot_core::{Connection, WindowSelector, WorkspaceRef};

impl Desktop {
    pub(super) async fn switch_workspace(&self, id: i32) -> Result<String> {
        let conn = self
            .connection
            .as_ref()
            .context("No Hyprland session; start hyprlume inside your own session")?;
        if uses_lua(conn).await? {
            conn.dispatch(&format!("hl.dsp.focus({{ workspace = {id} }})"))
                .await?;
        } else {
            conn.switch_workspace(&WorkspaceRef::Id(id)).await?;
        }
        if conn.active_workspace().await?.id != id {
            bail!("Hyprland did not confirm the requested workspace");
        }
        Ok(format!("Workspace {id} confirmed"))
    }

    pub(super) async fn move_window(&self, id: i32, target: Option<&str>) -> Result<String> {
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
            .any(|w| w.address == window.address && w.workspace.id == id);
        if !moved {
            bail!("Hyprland did not confirm the window move");
        }
        Ok(format!("{} moved to workspace {id}", window.class))
    }

    pub(super) async fn focus_window(&self, app: &str) -> Result<String> {
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
        focus_bound(conn, &target.address).await?;
        Ok(format!("{} focused", target.class))
    }

    pub async fn focus_address(&self, address: &str) -> Result<String> {
        lua_window(address)?;
        if self.demo {
            return Ok(format!("Demo only · focus {address}"));
        }
        let conn = self.connection.as_ref().context("No Hyprland session")?;
        let target = conn
            .clients()
            .await?
            .into_iter()
            .find(|window| window.address == address)
            .context("The selected window is no longer open")?;
        focus_bound(conn, &target.address).await?;
        Ok(format!("{} focused", target.class))
    }
}

async fn focus_bound(conn: &Connection, address: &str) -> Result<()> {
    let selector = lua_window(address)?;
    if uses_lua(conn).await? {
        conn.dispatch(&format!("hl.dsp.focus({{ window = {selector} }})"))
            .await?;
    } else {
        conn.focus_window(&WindowSelector::Address(address.to_owned()))
            .await?;
    }
    if !conn
        .active_window()
        .await?
        .is_some_and(|w| w.address == address)
    {
        bail!("Hyprland did not confirm the focused window");
    }
    Ok(())
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
