use anyhow::{ensure, Context, Result};
use std::{collections::HashMap, time::Duration};
use zbus::{
    names::OwnedUniqueName,
    zvariant::{OwnedObjectPath, OwnedValue},
    Connection, Proxy,
};

pub(super) const SERVICE: &str = "org.freedesktop.login1";
const SESSION: &str = "org.freedesktop.login1.Session";

pub(super) struct Login<'a> {
    pub properties: Proxy<'a>,
    pub id: String,
    timestamp: u64,
}

struct SessionSnapshot {
    id: String,
    timestamp: u64,
    active_wayland: bool,
    locked: bool,
}

impl Login<'_> {
    pub(super) async fn connect(connection: &Connection, owner: OwnedUniqueName) -> Result<Self> {
        let manager = Proxy::new(
            connection,
            owner.clone(),
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
        .await?;
        let hint = match std::env::var("XDG_SESSION_ID") {
            Ok(id) => {
                ensure!(valid_id(&id), "Invalid XDG_SESSION_ID for Polkit");
                Some(id)
            }
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => anyhow::bail!("Invalid XDG_SESSION_ID for Polkit"),
        };
        let managed = std::env::var("HYPRLUME_SESSION_READY").as_deref() == Ok("1")
            && std::env::var("XDG_CURRENT_DESKTOP")
                .is_ok_and(|desktop| desktop.split(':').any(|name| name == "Hyprlume"));
        let (properties, session) = resolve(&manager, owner, hint.as_deref(), managed).await?;
        ensure!(
            hint.is_none_or(|hint| hint == session.id),
            "XDG_SESSION_ID differs from the process's graphical login"
        );
        ensure!(
            session.active_wayland && !session.locked,
            "Polkit requires an active, unlocked Wayland login owned by this user"
        );
        Ok(Self {
            properties,
            id: session.id,
            timestamp: session.timestamp,
        })
    }

    pub(super) async fn available(&self) -> bool {
        !crate::services::locking::has_episode()
            && snapshot(&self.properties).await.is_ok_and(|session| {
                session.active_wayland
                    && !session.locked
                    && session.id == self.id
                    && session.timestamp == self.timestamp
            })
    }
}

async fn resolve<'a>(
    manager: &Proxy<'a>,
    owner: OwnedUniqueName,
    hint: Option<&str>,
    managed: bool,
) -> Result<(Proxy<'a>, SessionSnapshot)> {
    let path: OwnedObjectPath = match manager
        .call("GetSessionByPID", &(std::process::id(),))
        .await
    {
        Ok(path) => path,
        Err(zbus::Error::MethodError(name, _, _))
            if name.as_str() == "org.freedesktop.login1.NoSessionForPID" =>
        {
            if let Some(id) = hint {
                manager.call("GetSession", &(id,)).await?
            } else if managed {
                return managed_login(manager, &owner).await;
            } else {
                anyhow::bail!("Import XDG_SESSION_ID from your Wayland session");
            }
        }
        Err(error) => return Err(error).context("Resolve the graphical login"),
    };
    read_at(manager, &owner, path).await
}

async fn managed_login<'a>(
    manager: &Proxy<'a>,
    owner: &OwnedUniqueName,
) -> Result<(Proxy<'a>, SessionSnapshot)> {
    // UWSM session services run outside a logind session scope. Bind to the
    // unique active local Wayland login on a seat for this UID.
    let sessions: Vec<(String, u32, String, String, OwnedObjectPath)> =
        manager.call("ListSessions", &()).await?;
    let mut selected = None;
    for (id, uid, _, seat, path) in sessions {
        if uid != rustix::process::geteuid().as_raw() || seat.is_empty() {
            continue;
        }
        let candidate = read_at(manager, owner, path).await?;
        ensure!(id == candidate.1.id, "Logind session identity changed");
        if candidate.1.active_wayland {
            ensure!(
                selected.is_none(),
                "Multiple active Wayland logins; import XDG_SESSION_ID"
            );
            selected = Some(candidate);
        }
    }
    selected.context("No active local Wayland login for the managed Hyprlume session")
}

async fn read_at<'a>(
    manager: &Proxy<'a>,
    owner: &OwnedUniqueName,
    path: OwnedObjectPath,
) -> Result<(Proxy<'a>, SessionSnapshot)> {
    let properties = Proxy::new(
        manager.connection(),
        owner.clone(),
        path,
        "org.freedesktop.DBus.Properties",
    )
    .await?;
    let session = snapshot(&properties).await?;
    Ok((properties, session))
}

async fn snapshot(properties: &Proxy<'_>) -> Result<SessionSnapshot> {
    let values: HashMap<String, OwnedValue> = tokio::time::timeout(
        Duration::from_secs(2),
        properties.call("GetAll", &(SESSION,)),
    )
    .await??;
    let key = |name| {
        values
            .get(name)
            .with_context(|| format!("Missing logind session property {name}"))
    };
    let user: (u32, OwnedObjectPath) = key("User")?.try_clone()?.try_into()?;
    let id = <&str>::try_from(key("Id")?)?;
    let timestamp = u64::try_from(key("TimestampMonotonic")?)?;
    ensure!(
        valid_id(id) && timestamp > 0,
        "Logind did not identify the graphical login"
    );
    let active_wayland = user.0 == rustix::process::geteuid().as_raw()
        && !bool::try_from(key("Remote")?)?
        && bool::try_from(key("Active")?)?
        && <&str>::try_from(key("Type")?)? == "wayland"
        && matches!(<&str>::try_from(key("Class")?)?, "user" | "user-early")
        && <&str>::try_from(key("State")?)? != "closing";
    Ok(SessionSnapshot {
        id: id.into(),
        timestamp,
        active_wayland,
        locked: bool::try_from(key("LockedHint")?)?,
    })
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || b"_-".contains(&ch))
}
