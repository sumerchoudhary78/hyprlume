//! Standalone Polkit agent. The desktop daemon never receives PAM responses.
#[cfg(target_os = "linux")]
mod authority;
#[cfg(all(test, target_os = "linux"))]
mod bus_tests;
#[cfg(target_os = "linux")]
mod login;
#[cfg(target_os = "linux")]
mod pam;
#[cfg(target_os = "linux")]
mod wire;
#[cfg(target_os = "linux")]
mod worker;

use anyhow::{ensure, Context, Result};
use std::{os::unix::process::CommandExt, path::PathBuf, process::Command};

pub fn launch(path: Option<PathBuf>, socket: Option<PathBuf>) -> Result<()> {
    preflight()?;
    let path = path.unwrap_or_else(|| "/usr/share/hyprlume/shell/polkit.qml".into());
    let path = if path.is_dir() {
        path.join("polkit.qml")
    } else {
        path
    };
    ensure!(path.is_file(), "Polkit UI missing at {}", path.display());
    let mut command = Command::new("qs");
    command
        .args(["--path"])
        .arg(path)
        .env("QT_QPA_PLATFORM", "wayland")
        .env_remove("DISPLAY")
        .env_remove("POLKIT_DEBUG")
        .env("HYPRLUME_POLKIT_BIN", std::env::current_exe()?);
    if let Some(socket) = socket {
        command.env("HYPRLUME_SOCKET", socket);
    }
    Err(command.exec()).context("Start the Hyprlume Polkit dialog (install quickshell)")
}

fn preflight() -> Result<()> {
    ensure!(cfg!(target_os = "linux"), "The Polkit agent requires Linux");
    ensure!(
        std::env::var_os("HYPRLUME_PREVIEW").as_deref() != Some(std::ffi::OsStr::new("1")),
        "The Polkit agent is disabled in preview mode"
    );
    ensure!(
        !rustix::process::geteuid().is_root(),
        "Run the Polkit agent as your desktop user"
    );
    ensure!(
        std::env::var_os("WAYLAND_DISPLAY").is_some_and(|display| !display.is_empty()),
        "Start the Polkit agent from your Wayland session"
    );
    let status = std::fs::read_to_string("/proc/self/status")?;
    ensure!(
        status
            .lines()
            .any(|line| line.split_whitespace().eq(["NoNewPrivs:", "0"])),
        "Start the Polkit agent separately from the sandboxed Hyprlume daemon"
    );
    Ok(())
}

pub async fn worker() -> Result<()> {
    preflight()?;
    ensure!(
        std::env::var_os("POLKIT_DEBUG").is_none(),
        "Disable Polkit debug output before starting the authentication worker"
    );
    #[cfg(target_os = "linux")]
    return worker::run().await;
    #[cfg(not(target_os = "linux"))]
    anyhow::bail!("The Polkit agent requires Linux")
}
