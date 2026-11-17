use super::{Collection, Provider, Request, State};
use crate::services::locking::Controller;
use anyhow::{ensure, Context, Result};
use futures_util::StreamExt;
use std::time::Duration;
use zbus::{
    fdo::{DBusProxy, Error::NameHasNoOwner},
    names::OwnedUniqueName,
    proxy::CacheProperties,
    zvariant::{OwnedObjectPath, OwnedValue},
    Connection, Proxy,
};

const SERVICE: &str = "org.freedesktop.secrets";
const PATH: &str = "/org/freedesktop/secrets";
const INTERFACE: &str = "org.freedesktop.Secret.Service";

async fn owner(bus: &DBusProxy<'_>) -> Result<Option<OwnedUniqueName>> {
    match bus.get_name_owner(SERVICE.try_into()?).await {
        Ok(name) => Ok(Some(name)),
        Err(NameHasNoOwner(_)) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

async fn read_alias(service: &Proxy<'_>, alias: Collection) -> Result<Option<OwnedObjectPath>> {
    let alias = match alias {
        Collection::Login => "login",
        Collection::Default => "default",
    };
    let path: OwnedObjectPath = service.call("ReadAlias", &(alias,)).await?;
    Ok((path.as_str() != "/").then_some(path))
}

async fn locked(service: &Proxy<'_>, path: &OwnedObjectPath) -> Result<bool> {
    let proxy = zbus::proxy::Builder::<Proxy<'_>>::new(service.connection())
        .destination(service.destination().clone())?
        .path(path.as_str())?
        .interface("org.freedesktop.Secret.Collection")?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;
    Ok(proxy.get_property("Locked").await?)
}

async fn observe(service: &Proxy<'_>, alias: Collection) -> Result<Option<bool>> {
    match read_alias(service, alias).await? {
        Some(path) => Ok(Some(locked(service, &path).await?)),
        None => Ok(None),
    }
}

async fn prompt(
    service: &Proxy<'_>,
    path: &OwnedObjectPath,
    locking: &Controller,
    epoch: u64,
) -> Result<()> {
    let proxy = Proxy::new(
        service.connection(),
        service.destination().clone(),
        path.as_str(),
        "org.freedesktop.Secret.Prompt",
    )
    .await?;
    let mut completed = proxy.receive_signal("Completed").await?;
    let interaction = async {
        proxy.call::<_, _, ()>("Prompt", &("",)).await?;
        let signal = completed.next().await.context("Credential prompt closed")?;
        let (dismissed, _): (bool, OwnedValue) = signal.body().deserialize()?;
        ensure!(!dismissed, "Credential prompt was dismissed");
        Ok(())
    };
    let result = tokio::select! {
        biased;
        _ = locking.invalidated(epoch) => Err(anyhow::anyhow!("Session locking dismissed the credential prompt")),
        result = tokio::time::timeout(Duration::from_secs(110), interaction) => {
            result.context("Credential prompt timed out").and_then(|result| result)
        }
    };
    if result.is_err() {
        let _ = tokio::time::timeout(
            Duration::from_secs(1),
            proxy.call::<_, _, ()>("Dismiss", &()),
        )
        .await;
    }
    result
}

pub(super) async fn handle(locking: &Controller, epoch: u64, request: Request) -> Result<State> {
    let connection = Connection::session().await?;
    handle_on(&connection, locking, epoch, request).await
}

async fn handle_on(
    connection: &Connection,
    locking: &Controller,
    epoch: u64,
    request: Request,
) -> Result<State> {
    let bus = DBusProxy::new(connection).await?;
    if matches!(request, Request::Activate) && owner(&bus).await?.is_none() {
        ensure!(
            !locking.active() && locking.epoch() == epoch,
            "Session locking paused the credential action"
        );
        bus.start_service_by_name(SERVICE.try_into()?, 0)
            .await
            .context("Start the configured Secret Service provider")?;
    }
    let Some(provider) = owner(&bus).await? else {
        let activatable = bus
            .list_activatable_names()
            .await?
            .iter()
            .any(|name| name.as_str() == SERVICE);
        return Ok(State {
            provider: if activatable {
                Provider::OnDemand
            } else {
                Provider::Missing
            },
            message: if activatable {
                "A Secret Service provider is available on demand."
            } else {
                "Install a Secret Service provider, such as GNOME Keyring."
            }
            .into(),
            ..State::default()
        });
    };
    let service = Proxy::new(connection, provider.as_str(), PATH, INTERFACE).await?;
    let mut status = State {
        provider: Provider::Running,
        message: "Connected to the current Secret Service provider.".into(),
        ..State::default()
    };
    if let Request::Lock { collection } | Request::Unlock { collection } = request {
        let lock = matches!(request, Request::Lock { .. });
        let path = read_alias(&service, collection)
            .await?
            .context("The provider has no collection for this alias")?;
        ensure!(
            owner(&bus).await?.as_ref() == Some(&provider)
                && !locking.active()
                && locking.epoch() == epoch,
            "The session or credential provider changed"
        );
        let method = if lock { "Lock" } else { "Unlock" };
        let result: Result<()> = async {
            let (_, prompt_path): (Vec<OwnedObjectPath>, OwnedObjectPath) =
                service.call(method, &(vec![&path],)).await?;
            if prompt_path.as_str() != "/" {
                prompt(&service, &prompt_path, locking, epoch).await?;
            }
            ensure!(
                locked(&service, &path).await? == lock,
                "The provider did not confirm the requested state"
            );
            Ok(())
        }
        .await;
        if let Err(error) = result {
            // D-Bus errors can include provider-generated strings. Keep only
            // the local error context in the desktop and daemon logs.
            status.error = if error.is::<zbus::Error>() {
                "The credential provider declined the request".into()
            } else {
                error.to_string()
            };
        } else {
            status.message = if lock {
                "Collection locked."
            } else {
                "Collection unlocked."
            }
            .into();
        }
    }

    ensure!(
        owner(&bus).await?.as_ref() == Some(&provider),
        "The credential provider changed; refresh its status"
    );
    (status.login_locked, status.default_locked) = tokio::try_join!(
        observe(&service, Collection::Login),
        observe(&service, Collection::Default),
    )?;
    Ok(status)
}

#[cfg(test)]
mod tests;
