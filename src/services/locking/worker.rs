//! Authentication and ext-session-lock belong to the maintained Swaylock client.
//! This worker never reads a password, and never exposes a programmatic unlock.
use anyhow::{ensure, Context, Result};
use std::{
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    process::Stdio,
    time::Duration,
};
use tokio::{io::AsyncReadExt, process::Command, time::timeout};
pub(super) fn preflight() -> Result<()> {
    ensure!(cfg!(target_os = "linux"), "Session locking requires Linux");
    ensure!(
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
        "Session locking requires this daemon's Wayland session"
    );
    let executable = std::fs::metadata("/usr/bin/swaylock")
        .context("Install swaylock for secure session locking")?;
    ensure!(
        executable.is_file()
            && executable.uid() == 0
            && executable.permissions().mode() & 0o022 == 0,
        "Swaylock must be installed by the system package manager"
    );
    let policy = std::fs::metadata("/etc/pam.d/swaylock")
        .context("Install Swaylock's system PAM policy before locking")?;
    ensure!(
        policy.is_file() && policy.uid() == 0 && policy.permissions().mode() & 0o022 == 0,
        "The Swaylock PAM policy must be root-owned and not writable by other users"
    );
    Ok(())
}
fn background() -> Option<PathBuf> {
    let mut candidates = vec![PathBuf::from(
        "/usr/share/hyprlume/shell/assets/backgrounds/lock.svg",
    )];
    if let Ok(executable) = std::env::current_exe() {
        if let Some(prefix) = executable.parent().and_then(|bin| bin.parent()) {
            candidates.push(prefix.join("share/hyprlume/shell/assets/backgrounds/lock.svg"));
        }
    }
    candidates
        .push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shell/assets/backgrounds/lock.svg"));
    candidates.into_iter().find(|path| path.is_file())
}
pub(super) fn client_args() -> Vec<String> {
    let mut args = vec![
        "--config",
        "/dev/null",
        "--ready-fd",
        "1",
        "--ignore-empty-password",
        "--show-failed-attempts",
        "--indicator-idle-visible",
        "--indicator-caps-lock",
        "--color",
        "030914",
        "--scaling",
        "fill",
        "--font",
        "Inter",
        "--indicator-radius",
        "62",
        "--indicator-thickness",
        "4",
        "--inside-color",
        "071526df",
        "--ring-color",
        "5ce1ffcc",
        "--line-color",
        "00000000",
        "--key-hl-color",
        "b497ff",
        "--bs-hl-color",
        "ffb66e",
        "--text-color",
        "edf4ff",
        "--ring-ver-color",
        "68f0b1",
        "--ring-wrong-color",
        "ff8f9e",
        "--inside-wrong-color",
        "341326ed",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    if let Some(path) = background() {
        args.push("--image".into());
        args.push(format!(":{}", path.display()));
    }
    args
}
/// Internal systemd worker. Readiness is forwarded only after Swaylock's
/// compositor acknowledgment. A shell/daemon restart cannot kill this unit.
pub async fn run() -> Result<()> {
    preflight()?;
    ensure!(
        std::env::var_os("NOTIFY_SOCKET").is_some(),
        "The lock worker must run in its dedicated systemd notify unit"
    );
    let version = timeout(
        Duration::from_secs(2),
        Command::new("/usr/bin/swaylock").arg("--version").output(),
    )
    .await
    .context("Swaylock version check timed out")??;
    ensure!(
        version.status.success() && supported_version(&String::from_utf8_lossy(&version.stdout)),
        "Swaylock 1.6 or newer with ext-session-lock is required"
    );
    super::marker::begin()?;
    let mut child = Command::new("/usr/bin/env")
        .args(["--block-signal=USR1", "--", "/usr/bin/swaylock"])
        .args(client_args())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("Start secure Wayland lock client")?;
    let mut ready = [0u8; 1];
    timeout(
        Duration::from_secs(15),
        child
            .stdout
            .take()
            .context("Lock readiness pipe unavailable")?
            .read_exact(&mut ready),
    )
    .await
    .context("Compositor did not acknowledge the lock in time")??;
    ensure!(ready == *b"\n", "Invalid compositor lock readiness");
    let notified = Command::new("/usr/bin/systemd-notify")
        .args([
            "--ready",
            "--status=Session locked; PAM authentication required",
        ])
        .arg(format!("--pid={}", std::process::id()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status();
    ensure!(
        timeout(Duration::from_secs(5), notified)
            .await
            .context("Lock readiness notification timed out")??
            .success(),
        "Could not publish lock readiness"
    );
    let status = child.wait().await?;
    ensure!(
        status.success(),
        "Lock client failed; the compositor may still require session recovery"
    );
    super::marker::completed()?;
    Ok(())
}

fn supported_version(value: &str) -> bool {
    value.split_whitespace().any(|part| {
        let mut numbers = part.split('.');
        let major = numbers.next().and_then(|part| part.parse::<u32>().ok());
        let minor = numbers.next().and_then(|part| part.parse::<u32>().ok());
        matches!((major,minor),(Some(major),Some(minor)) if major>1 || (major==1 && minor>=6))
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn older_input_inhibitor_clients_are_refused() {
        assert!(supported_version(
            "swaylock version 1.8.6
"
        ));
        assert!(supported_version("swaylock version 1.6.0"));
        assert!(!supported_version("swaylock version 1.5.1"));
        assert!(!supported_version("unknown"));
    }
}
