//! A maintained Wayland/logind event source; policy stays in Rust.
use super::{Credential, Event};
use crate::{config::Idle, protocol::Operation};
use anyhow::{ensure, Context, Result};
use std::{io::Read, path::Path, process::Stdio};
use tokio::process::{Child, Command};

pub(super) fn credential() -> Result<Credential> {
    let mut bytes = [0u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(Credential(
        bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
    ))
}

fn arguments(policy: &Idle, paused: bool) -> Vec<String> {
    let mut args: Vec<String> = ["-w", "-C", "/dev/null"].map(String::from).into();
    // Fixed commands only. The quoted executable is supplied by this process,
    // never assembled from settings, desktop entries or model output.
    for (event, command) in [
        (
            "before-sleep",
            "exec \"$HYPRLUME_IDLE_EXE\" idle-event before-sleep",
        ),
        (
            "after-resume",
            "exec \"$HYPRLUME_IDLE_EXE\" idle-event after-resume",
        ),
        ("lock", "exec \"$HYPRLUME_IDLE_EXE\" idle-event force-lock"),
    ] {
        args.extend([event.into(), command.into()]);
    }
    if !paused {
        for (seconds, command) in [
            (
                policy.dim_after_seconds,
                "exec \"$HYPRLUME_IDLE_EXE\" idle-event dim",
            ),
            (
                policy.lock_after_seconds,
                "exec \"$HYPRLUME_IDLE_EXE\" idle-event lock",
            ),
            (
                policy.screen_off_after_seconds,
                "exec \"$HYPRLUME_IDLE_EXE\" idle-event off",
            ),
            (
                policy.suspend_after_seconds,
                "exec \"$HYPRLUME_IDLE_EXE\" idle-event suspend",
            ),
        ] {
            if seconds > 0 {
                args.extend([
                    "timeout".into(),
                    seconds.to_string(),
                    command.into(),
                    "resume".into(),
                    "exec \"$HYPRLUME_IDLE_EXE\" idle-event resume".into(),
                ]);
            }
        }
    }
    args
}

pub(super) fn start(
    socket: &Path,
    token: &Credential,
    policy: &Idle,
    paused: bool,
) -> Result<Child> {
    ensure!(
        cfg!(target_os = "linux"),
        "Idle management requires Linux and Wayland"
    );
    ensure!(
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
        "No Wayland display is available"
    );
    policy.validate()?;
    Command::new("swayidle")
        .args(arguments(policy, paused))
        .env("HYPRLUME_IDLE_EXE", std::env::current_exe()?)
        .env("HYPRLUME_IDLE_SOCKET", socket)
        .env("HYPRLUME_IDLE_TOKEN", &token.0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("Start swayidle (install swayidle with logind support)")
}

pub async fn callback(event: Event) -> Result<()> {
    let socket =
        std::env::var_os("HYPRLUME_IDLE_SOCKET").context("Not an idle provider callback")?;
    let credential =
        Credential(std::env::var("HYPRLUME_IDLE_TOKEN").context("Missing idle capability")?);
    crate::server::call(
        Path::new(&socket),
        Operation::IdleEvent { credential, event },
    )
    .await?;
    if matches!(event, Event::BeforeSleep | Event::ForceLock) {
        crate::server::wait_for_lock(Path::new(&socket)).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paused_provider_keeps_sleep_hooks_but_has_no_idle_timers() {
        let policy = Idle::default();
        let paused = arguments(&policy, true);
        assert!(!paused.iter().any(|a| a == "timeout"));
        assert!(paused.iter().any(|a| a == "before-sleep"));
        let running = arguments(&policy, false);
        assert_eq!(running.iter().filter(|a| *a == "timeout").count(), 3);
        assert!(!running.iter().any(|a| a.contains("idle-event suspend")));
        assert!(!format!("{:?}", credential().unwrap()).contains("Credential"));
    }
}
