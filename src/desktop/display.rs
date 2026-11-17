//! Exact DPMS actions with a bounded restore lease; no arbitrary dispatch strings.
use super::{hyprland::uses_lua, Desktop};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct Display {
    name: String,
    id: i32,
    description: String,
    #[serde(rename = "dpmsStatus")]
    powered: bool,
}

impl Display {
    fn same(&self, other: &Self) -> bool {
        self.name == other.name && self.id == other.id && self.description == other.description
    }
}
impl Desktop {
    async fn displays(&self) -> Result<Vec<Display>> {
        let conn = self
            .connection
            .as_ref()
            .context("Display power needs the current Hyprland session")?;
        let displays: Vec<Display> = serde_json::from_str(&conn.send_raw("j/monitors").await?)?;
        ensure!(displays.len() <= 32, "Too many displays");
        Ok(displays)
    }
    async fn display_power(&self, display: &Display, on: bool) -> Result<()> {
        ensure!(!self.demo, "Display power is disabled in demo mode");
        ensure!(
            !display.name.is_empty()
                && display.name.len() <= 128
                && display
                    .name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c)),
            "Unsupported monitor name"
        );
        let conn = self.connection.as_ref().context("No Hyprland session")?;
        if uses_lua(conn).await? {
            let name = serde_json::to_string(&display.name)?;
            let action = if on { "enable" } else { "disable" };
            conn.dispatch(&format!(
                "hl.dsp.dpms({{ action = \"{action}\", monitor = {name} }})"
            ))
            .await?;
        } else {
            conn.dispatch(&format!(
                "dpms {} {}",
                if on { "on" } else { "off" },
                display.name
            ))
            .await?;
        }
        ensure!(
            self.displays()
                .await?
                .iter()
                .any(|d| d.same(display) && d.powered == on),
            "Hyprland did not confirm display power"
        );
        Ok(())
    }
    pub(crate) async fn blank_displays(&self, lease: &mut Vec<Display>) -> Result<()> {
        for display in self.displays().await?.into_iter().filter(|d| d.powered) {
            lease.push(display.clone());
            self.display_power(&display, false).await?;
        }
        Ok(())
    }
    pub(crate) async fn restore_displays(&self, lease: &mut Vec<Display>) -> Result<()> {
        if lease.is_empty() {
            return Ok(());
        }
        let current = self.displays().await?;
        let mut error = None;
        for display in lease.iter() {
            if current.iter().any(|d| d.same(display) && !d.powered) {
                if let Err(e) = self.display_power(display, true).await {
                    error = Some(e);
                }
            }
        }
        lease.clear();
        if let Some(error) = error {
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyprpilot_core::{Connection, Instance};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixListener,
    };
    async fn fixture(
        steps: Vec<(&str, String)>,
    ) -> (tempfile::TempDir, Desktop, tokio::task::JoinHandle<()>) {
        let dir = tempfile::tempdir().unwrap();
        let instance = Instance {
            signature: "idle-test".into(),
            runtime_dir: dir.path().into(),
        };
        std::fs::create_dir_all(instance.control_socket().parent().unwrap()).unwrap();
        let listener = UnixListener::bind(instance.control_socket()).unwrap();
        let steps: Vec<_> = steps.into_iter().map(|(q, a)| (q.to_owned(), a)).collect();
        let server = tokio::spawn(async move {
            for (query, answer) in steps {
                let (mut socket, _) =
                    tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept())
                        .await
                        .unwrap()
                        .unwrap();
                let mut input = String::new();
                socket.read_to_string(&mut input).await.unwrap();
                assert_eq!(input, query);
                socket.write_all(answer.as_bytes()).await.unwrap();
            }
        });
        (
            dir,
            Desktop {
                connection: Some(Connection::with_instance(instance)),
                demo: false,
            },
            server,
        )
    }
    fn monitors(on: bool, id: i32) -> String {
        serde_json::json!([{"name":"eDP-1","id":id,"description":"Fixture panel","dpmsStatus":on}])
            .to_string()
    }
    #[tokio::test]
    async fn classic_and_lua_dpms_use_exact_actions_and_verify_observed_power() {
        for lua in [false, true] {
            let off = if lua {
                "dispatch hl.dsp.dpms({ action = \"disable\", monitor = \"eDP-1\" })"
            } else {
                "dispatch dpms off eDP-1"
            };
            let on = if lua {
                "dispatch hl.dsp.dpms({ action = \"enable\", monitor = \"eDP-1\" })"
            } else {
                "dispatch dpms on eDP-1"
            };
            let status =
                serde_json::json!({"configProvider":if lua {"lua"} else {"hyprlang"}}).to_string();
            let (_dir, desktop, server) = fixture(vec![
                ("j/monitors", monitors(true, 1)),
                ("j/status", status.clone()),
                (off, "ok".into()),
                ("j/monitors", monitors(false, 1)),
                ("j/monitors", monitors(false, 1)),
                ("j/status", status),
                (on, "ok".into()),
                ("j/monitors", monitors(true, 1)),
            ])
            .await;
            let mut lease = Vec::new();
            desktop.blank_displays(&mut lease).await.unwrap();
            assert_eq!(lease.len(), 1);
            desktop.restore_displays(&mut lease).await.unwrap();
            assert!(lease.is_empty());
            server.await.unwrap();
        }
    }
    #[tokio::test]
    async fn failed_confirmation_keeps_a_restore_lease_and_a_replacement_output_is_not_touched() {
        let (_dir, desktop, server) = fixture(vec![
            ("j/monitors", monitors(true, 1)),
            ("j/status", "unknown request".into()),
            ("dispatch dpms off eDP-1", "ok".into()),
            ("j/monitors", monitors(true, 1)),
            ("j/monitors", monitors(false, 2)),
        ])
        .await;
        let mut lease = Vec::new();
        assert!(desktop.blank_displays(&mut lease).await.is_err());
        assert_eq!(lease.len(), 1);
        desktop.restore_displays(&mut lease).await.unwrap();
        assert!(lease.is_empty());
        server.await.unwrap();
    }
}
