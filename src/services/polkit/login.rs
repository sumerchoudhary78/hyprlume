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
        let path: OwnedObjectPath = match manager
            .call("GetSessionByPID", &(std::process::id(),))
            .await
        {
            Ok(path) => path,
            Err(error) => {
                let id = hint.as_ref().with_context(|| format!("Could not resolve the graphical login ({error}). Import XDG_SESSION_ID from your Wayland session."))?;
                manager.call("GetSession", &(id,)).await?
            }
        };
        let properties =
            Proxy::new(connection, owner, path, "org.freedesktop.DBus.Properties").await?;
        let (id, timestamp, allowed) = snapshot(&properties).await?;
        ensure!(
            hint.is_none_or(|hint| hint == id),
            "XDG_SESSION_ID differs from the process's graphical login"
        );
        ensure!(
            allowed,
            "Polkit requires an active, unlocked Wayland login owned by this user"
        );
        Ok(Self {
            properties,
            id,
            timestamp,
        })
    }

    pub(super) async fn available(&self) -> bool {
        !crate::services::locking::has_episode()
            && snapshot(&self.properties)
                .await
                .is_ok_and(|(id, timestamp, allowed)| {
                    allowed && id == self.id && timestamp == self.timestamp
                })
    }
}

async fn snapshot(properties: &Proxy<'_>) -> Result<(String, u64, bool)> {
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
    let allowed = user.0 == rustix::process::geteuid().as_raw()
        && !bool::try_from(key("Remote")?)?
        && bool::try_from(key("Active")?)?
        && !bool::try_from(key("LockedHint")?)?
        && <&str>::try_from(key("Type")?)? == "wayland"
        && matches!(<&str>::try_from(key("Class")?)?, "user" | "user-early")
        && <&str>::try_from(key("State")?)? != "closing";
    Ok((id.into(), timestamp, allowed))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || b"_-".contains(&ch))
}
