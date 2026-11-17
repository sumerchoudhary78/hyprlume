use super::{BusState, Portal, Status, APPEARANCE_BACKEND, FRONTEND, PATH};
use anyhow::Result;
use futures_util::future::join_all;
use zbus::{fdo::DBusProxy, proxy::CacheProperties, zvariant::OwnedValue, Connection, Proxy};

const PORTALS: [(&str, &str); 5] = [
    ("File dialogs", "org.freedesktop.portal.FileChooser"),
    ("Screen sharing", "org.freedesktop.portal.ScreenCast"),
    ("Screenshots", "org.freedesktop.portal.Screenshot"),
    ("App shortcuts", "org.freedesktop.portal.GlobalShortcuts"),
    ("Appearance", "org.freedesktop.portal.Settings"),
];

pub(super) async fn inspect(snapshot: &mut Status) -> Result<()> {
    let connection = Connection::session().await?;
    let bus = DBusProxy::new(&connection).await?;
    let (owners, activatable) = tokio::try_join!(bus.list_names(), bus.list_activatable_names())?;
    let state = |name| {
        if owners.iter().any(|owner| owner.as_str() == name) {
            BusState::Running
        } else if activatable.iter().any(|service| service.as_str() == name) {
            BusState::OnDemand
        } else {
            BusState::Missing
        }
    };
    snapshot.frontend = state(FRONTEND);
    snapshot.hyprland = state("org.freedesktop.impl.portal.desktop.hyprland");
    snapshot.gtk = state("org.freedesktop.impl.portal.desktop.gtk");
    snapshot.appearance = state(APPEARANCE_BACKEND);
    snapshot.message = match snapshot.frontend {
        BusState::Running => "Interfaces reported by the current portal service.",
        BusState::OnDemand => {
            "The portal service can start on demand when an application needs it."
        }
        BusState::Missing => "The portal service is missing from this session bus.",
        BusState::Unknown => unreachable!(),
    }
    .into();
    if let BusState::Running = snapshot.frontend {
        // Bind property reads to the current owner. These checks never activate a portal.
        let owner = bus.get_name_owner(FRONTEND.try_into()?).await?;
        let properties = zbus::proxy::Builder::<Proxy<'_>>::new(&connection)
            .destination(owner)?
            .path(PATH)?
            .interface("org.freedesktop.DBus.Properties")?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        snapshot.interfaces = join_all(PORTALS.into_iter().map(|(label, interface)| {
            let properties = &properties;
            async move {
                let version = properties
                    .call::<_, _, OwnedValue>("Get", &(interface, "version"))
                    .await
                    .ok()
                    .and_then(|value| u32::try_from(value).ok());
                Portal { label, version }
            }
        }))
        .await;
    }
    Ok(())
}
