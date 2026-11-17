//! Session portal diagnostics and the read-only appearance backend.
#[cfg(target_os = "linux")]
mod appearance;
#[cfg(target_os = "linux")]
mod feed;
#[cfg(target_os = "linux")]
mod status;

use serde::Serialize;
use std::path::PathBuf;

pub const PROFILE: &str = include_str!("../../../packaging/portals/hyprlume-portals.conf");
#[cfg(target_os = "linux")]
const FRONTEND: &str = "org.freedesktop.portal.Desktop";
#[cfg(target_os = "linux")]
const PATH: &str = "/org/freedesktop/portal/desktop";
#[cfg(target_os = "linux")]
const APPEARANCE_BACKEND: &str = "org.freedesktop.impl.portal.desktop.hyprlume";

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BusState {
    #[default]
    Unknown,
    Running,
    OnDemand,
    Missing,
}

#[derive(Debug, Default, Serialize)]
pub struct Status {
    pub desktop: String,
    pub profile_requested: bool,
    pub frontend: BusState,
    pub hyprland: BusState,
    pub gtk: BusState,
    pub appearance: BusState,
    pub interfaces: Vec<Portal>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Portal {
    pub label: &'static str,
    pub version: Option<u32>,
}

pub async fn inspect(demo: bool) -> Status {
    let mut snapshot = Status {
        desktop: std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .chars()
            .filter(|ch| !ch.is_control())
            .take(160)
            .collect(),
        ..Status::default()
    };
    snapshot.profile_requested = snapshot
        .desktop
        .split(':')
        .next()
        .is_some_and(|desktop| desktop.eq_ignore_ascii_case("hyprlume"));
    if demo {
        snapshot.message = "Portal checks are disabled in demo mode.".into();
        return snapshot;
    }
    #[cfg(target_os = "linux")]
    {
        use std::time::Duration;
        match tokio::time::timeout(Duration::from_secs(2), status::inspect(&mut snapshot)).await {
            Ok(Ok(())) => (),
            Ok(Err(_)) => {
                snapshot.message = "Could not read the session D-Bus portal state.".into()
            }
            Err(_) => snapshot.message = "The session D-Bus portal check timed out.".into(),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        snapshot.message = "Desktop portals require a Linux session bus.".into();
    }
    snapshot
}

pub async fn serve_appearance(config: PathBuf, socket: Option<PathBuf>) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let initial = crate::config::Config::load(Some(config))
            .unwrap_or_default()
            .preferences
            .appearance;
        let socket = socket
            .or_else(|| std::env::var_os("HYPRLUME_SOCKET").map(PathBuf::from))
            .or_else(|| crate::config::socket_path().ok());
        appearance::serve(initial, socket).await
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (config, socket);
        anyhow::bail!("Desktop portals require Linux")
    }
}
