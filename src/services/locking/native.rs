use super::Controller;
use anyhow::{ensure, Context, Result};
use futures_util::StreamExt;
use std::{process::Stdio, sync::Arc, time::Duration};
use tokio::{process::Command, time::timeout};
use zbus::{message::Type, zvariant::OwnedObjectPath, Connection, MatchRule, MessageStream, Proxy};
const SERVICE: &str = "org.freedesktop.systemd1";
const UNIT: &str = "hyprlume-lock.service";
pub(super) async fn start() -> Result<()> {
    let mut command = Command::new("/usr/bin/systemd-run");
    command.args([
        "--user",
        "--quiet",
        "--collect",
        "--service-type=notify",
        "--unit",
        UNIT,
        "--property=NotifyAccess=all",
        "--property=PartOf=graphical-session.target",
        "--property=NoNewPrivileges=no",
        "--property=Restart=on-failure",
        "--property=RestartSec=1",
        "--property=StartLimitIntervalSec=0",
        "--property=TimeoutStartSec=20",
        "--property=UMask=0077",
    ]);
    for name in [
        "WAYLAND_DISPLAY",
        "XDG_RUNTIME_DIR",
        "XDG_SESSION_TYPE",
        "XDG_SESSION_ID",
        "DBUS_SESSION_BUS_ADDRESS",
        "LANG",
        "LC_ALL",
    ] {
        if std::env::var_os(name).is_some() {
            command.arg(format!("--setenv={name}"));
        }
    }
    // The manager owns this unit independently. In particular, this worker
    // must not inherit the daemon's NoNewPrivileges PAM restriction.
    command
        .arg("--")
        .arg(std::env::current_exe()?)
        .arg("lock-worker");
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(false)
        .spawn()
        .context("Start dedicated lock service")?;
    let status = timeout(Duration::from_secs(25), child.wait_with_output())
        .await
        .context("Lock service readiness timed out; the compositor may still be protected")??;
    ensure!(status.status.success(),"The systemd user manager could not confirm locking; check the hyprlume-lock service journal");
    Ok(())
}
async fn unit_state(connection: &Connection) -> Result<String> {
    let manager = Proxy::new(
        connection,
        SERVICE,
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    )
    .await?;
    let path: Result<OwnedObjectPath, _> = manager.call("GetUnit", &(UNIT,)).await;
    let path = match path {
        Ok(path) => path,
        Err(zbus::Error::MethodError(name, _, _))
            if name.as_str() == "org.freedesktop.systemd1.NoSuchUnit" =>
        {
            return Ok("inactive".into())
        }
        Err(error) => return Err(error.into()),
    };
    let unit = Proxy::new(connection, SERVICE, path, "org.freedesktop.systemd1.Unit").await?;
    Ok(unit.get_property::<String>("ActiveState").await?)
}
pub(super) async fn observe(controller: Arc<Controller>) -> Result<()> {
    let connection = Connection::session().await?;
    let manager = Proxy::new(
        &connection,
        SERVICE,
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    )
    .await?;
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender(SERVICE)?
        .build();
    let mut events = MessageStream::for_match_rule(rule, &connection, Some(32)).await?;
    let _: () = manager.call("Subscribe", &()).await?;
    controller.observed(&unit_state(&connection).await?);
    while let Some(message) = events.next().await {
        let message = message?;
        let header = message.header();
        let relevant = header
            .path()
            .is_some_and(|path| path.as_str().ends_with("/hyprlume_2dlock_2eservice"))
            || match header.member().map(|name| name.as_str()) {
                Some("UnitNew" | "UnitRemoved") => message
                    .body()
                    .deserialize::<(String, OwnedObjectPath)>()
                    .is_ok_and(|(name, _)| name == UNIT),
                _ => false,
            };
        if relevant {
            controller.observed(&unit_state(&connection).await?);
        }
    }
    anyhow::bail!("Lock service event stream ended")
}

pub(super) async fn refresh(controller: &Controller) -> Result<()> {
    let connection = Connection::session().await?;
    controller.observed(&unit_state(&connection).await?);
    Ok(())
}
