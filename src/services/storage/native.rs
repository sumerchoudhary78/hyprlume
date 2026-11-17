mod catalog;

use super::{Request, State};
use anyhow::{ensure, Context, Result};
use catalog::{Device, Entry, CRYPT, FS};
use futures_util::StreamExt;
use std::{collections::HashMap, process::Stdio, time::Duration};
use zbus::{
    fdo::{DBusProxy, ObjectManagerProxy},
    message::Type,
    names::OwnedUniqueName,
    zvariant::{OwnedObjectPath, Value},
    Connection, MatchRule, MessageStream, Proxy,
};

const SERVICE: &str = "org.freedesktop.UDisks2";
const ROOT: &str = "/org/freedesktop/UDisks2";
type Options = HashMap<&'static str, Value<'static>>;

pub(crate) async fn connect() -> Result<Connection> {
    let connection = Connection::system().await?;
    let bus = DBusProxy::new(&connection).await?;
    if !bus.name_has_owner(SERVICE.try_into()?).await? {
        // Packaged UDisks2 is normally D-Bus activated on first use.
        let _ = bus.start_service_by_name(SERVICE.try_into()?, 0).await;
    }
    Ok(connection)
}
async fn owner(connection: &Connection) -> Result<OwnedUniqueName> {
    DBusProxy::new(connection)
        .await?
        .get_name_owner(SERVICE.try_into()?)
        .await
        .context("UDisks2 is unavailable; install or start the udisks2 service")
}
async fn scan(connection: &Connection, owner: &OwnedUniqueName) -> Result<Vec<Device>> {
    let proxy = ObjectManagerProxy::builder(connection)
        .destination(owner.as_str())?
        .path(ROOT)?
        .build()
        .await?;
    let objects = proxy
        .get_managed_objects()
        .await
        .context("Read removable drives")?;
    ensure!(objects.len() <= 512, "Too many storage devices");
    Ok(catalog::scan(&objects, owner.as_str()))
}

pub(crate) async fn read(connection: &Connection) -> Result<State> {
    let provider = owner(connection).await?;
    let drives = scan(connection, &provider).await?;
    Ok(State {
        available: true,
        message: if drives.is_empty() {
            "Connect a USB drive to get started."
        } else {
            "Removable drives are ready."
        }
        .into(),
        drives: drives.into_iter().take(8).map(|drive| drive.row).collect(),
        ..State::default()
    })
}

pub(crate) async fn watch(connection: &Connection) -> Result<(MessageStream, MessageStream)> {
    let signals = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender(SERVICE)?
        .path_namespace(ROOT)?
        .build();
    let owners = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender("org.freedesktop.DBus")?
        .interface("org.freedesktop.DBus")?
        .member("NameOwnerChanged")?
        .arg(0, SERVICE)?
        .build();
    Ok((
        MessageStream::for_match_rule(signals, connection, Some(12)).await?,
        MessageStream::for_match_rule(owners, connection, Some(4)).await?,
    ))
}

pub(crate) async fn changes(signals: &mut MessageStream, owners: &mut MessageStream) -> Result<()> {
    let event = tokio::select! {
        message = signals.next() => message,
        message = owners.next() => message,
    };
    event.context("Storage events disconnected")??;
    Ok(())
}

async fn proxy<'a>(
    connection: &'a Connection,
    owner: &'a OwnedUniqueName,
    path: &'a OwnedObjectPath,
    interface: &'static str,
) -> Result<Proxy<'a>> {
    Ok(Proxy::new(connection, owner.as_str(), path.as_str(), interface).await?)
}
fn volume<'a>(drives: &'a [Device], id: &str) -> Result<&'a Entry> {
    let entry = drives
        .iter()
        .flat_map(|drive| &drive.volumes)
        .find(|entry| entry.row.id == id)
        .context("This removable volume has changed; refresh and select it again")?;
    ensure!(
        !entry.row.protected,
        "This volume has a system or unmanaged mount"
    );
    Ok(entry)
}
async fn unmount(connection: &Connection, owner: &OwnedUniqueName, entry: &Entry) -> Result<()> {
    let path = entry
        .filesystem
        .as_ref()
        .context("The volume has no filesystem")?;
    proxy(connection, owner, path, FS)
        .await?
        .call::<_, _, ()>("Unmount", &(Options::new(),))
        .await
        .context("Unmount failed. Close files on the drive and try again.")?;
    Ok(())
}
async fn lock(connection: &Connection, owner: &OwnedUniqueName, entry: &Entry) -> Result<()> {
    ensure!(
        entry.row.encrypted && !entry.row.locked && entry.mounts.is_empty(),
        "Unmount the unlocked volume before locking it"
    );
    proxy(connection, owner, &entry.block, CRYPT)
        .await?
        .call::<_, _, ()>("Lock", &(Options::new(),))
        .await
        .context("Could not lock the encrypted volume")?;
    Ok(())
}
async fn eject(connection: &Connection, provider: &OwnedUniqueName, id: &str) -> Result<()> {
    // Refresh the drive and all its filesystems before each step. The service
    // refuses busy devices and a failed step stops the removal sequence.
    for _ in 0..34 {
        ensure!(
            owner(connection).await? == *provider,
            "The storage service changed"
        );
        let drives = scan(connection, provider).await?;
        let drive = drives
            .iter()
            .find(|drive| drive.row.id == id)
            .context("This removable drive has changed; refresh and select it again")?;
        let method = drive
            .removal
            .filter(|_| !drive.protected)
            .context("This drive does not support safe removal with its current mounts")?;
        if let Some(entry) = drive.volumes.iter().find(|entry| !entry.mounts.is_empty()) {
            unmount(connection, provider, entry).await?;
        } else if let Some(entry) = drive
            .volumes
            .iter()
            .find(|entry| entry.row.encrypted && !entry.row.locked)
        {
            lock(connection, provider, entry).await?;
        } else {
            proxy(
                connection,
                provider,
                &drive.path,
                "org.freedesktop.UDisks2.Drive",
            )
            .await?
            .call::<_, _, ()>(method, &(Options::new(),))
            .await
            .context("UDisks2 could not eject or power off this drive")?;
            return Ok(());
        }
    }
    anyhow::bail!("The drive still has active volumes. Refresh before trying again.")
}
pub(crate) async fn apply(connection: &Connection, request: Request) -> Result<&'static str> {
    let provider = owner(connection).await?;
    let id = match &request {
        Request::Refresh => return Ok("Drive list refreshed."),
        Request::Eject { id } => {
            eject(connection, &provider, id).await?;
            return Ok("Safe to remove the drive.");
        }
        Request::Mount { id }
        | Request::Unmount { id }
        | Request::Open { id }
        | Request::Unlock { id, .. }
        | Request::Lock { id } => id,
    };
    let devices = scan(connection, &provider).await?;
    let entry = volume(&devices, id)?;
    ensure!(
        owner(connection).await? == provider,
        "The storage service changed"
    );
    match request {
        Request::Mount { .. } => {
            ensure!(!entry.row.mounted, "The volume is already mounted");
            let path = entry
                .filesystem
                .as_ref()
                .context("The volume has no mountable filesystem")?;
            let _: String = proxy(connection, &provider, path, FS)
                .await?
                .call("Mount", &(Options::new(),))
                .await
                .context("UDisks2 could not mount the volume")?;
            Ok("Volume mounted.")
        }
        Request::Unmount { .. } => {
            ensure!(entry.row.mounted, "The volume is already unmounted");
            unmount(connection, &provider, entry).await?;
            Ok("Volume unmounted.")
        }
        Request::Open { .. } => {
            let path = entry
                .mounts
                .first()
                .context("Mount the volume before opening it")?;
            let mut command = tokio::process::Command::new("gio");
            command
                .args(["open", "--"])
                .arg(path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true);
            let opened = tokio::time::timeout(Duration::from_secs(5), command.status())
                .await
                .context("The file manager did not respond")?
                .context("Install GIO and configure a file manager")?;
            ensure!(
                opened.success(),
                "Could not open the mount point in the default file manager"
            );
            Ok("Opened in the file manager.")
        }
        Request::Unlock { password, .. } => {
            ensure!(
                entry.row.encrypted && entry.row.locked,
                "The encrypted volume is already unlocked"
            );
            let password = password.expose();
            ensure!(
                !password.is_empty() && password.len() <= 1024 && !password.contains('\0'),
                "Enter a passphrase of 1–1024 bytes"
            );
            let _: OwnedObjectPath = proxy(connection, &provider, &entry.block, CRYPT)
                .await?
                .call("Unlock", &(password, Options::new()))
                .await
                .context("Could not unlock the drive. Check the passphrase and permissions.")?;
            Ok("Encrypted volume unlocked.")
        }
        Request::Lock { .. } => {
            lock(connection, &provider, entry).await?;
            Ok("Encrypted volume locked.")
        }
        Request::Refresh | Request::Eject { .. } => unreachable!(),
    }
}

#[cfg(test)]
mod tests;
