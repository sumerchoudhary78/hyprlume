mod backlight;
mod battery;
mod profiles;
mod session;

use super::{types::Binding, Backend, Backlight, Controller, Hardware, Mutation};
use anyhow::{Context, Result};
use futures_util::{future::BoxFuture, stream::SelectAll, StreamExt};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tokio::sync::Semaphore;
use zbus::{message::Type, zvariant::OwnedValue, Connection, MatchRule, MessageStream, Proxy};

type Properties = HashMap<String, OwnedValue>;
const UPOWER: &str = "org.freedesktop.UPower";
const LOGIND: &str = "org.freedesktop.login1";
const PROFILES: &str = "org.freedesktop.UPower.PowerProfiles";
const LEGACY_PROFILES: &str = "net.hadess.PowerProfiles";

pub(super) struct Native {
    connection: Connection,
    backlight_root: PathBuf,
    session_hint: Option<String>,
    uid: u32,
    backlight_io: Arc<Semaphore>,
}
impl Native {
    pub(super) fn new(
        connection: Connection,
        backlight_root: PathBuf,
        session_hint: Option<String>,
        uid: u32,
    ) -> Self {
        Self {
            connection,
            backlight_root,
            session_hint,
            uid,
            backlight_io: Arc::new(Semaphore::new(1)),
        }
    }
    async fn binding(&self, service: &str, path: &str, interface: &str) -> Result<Binding> {
        let dbus = zbus::fdo::DBusProxy::new(&self.connection).await?;
        let owner = dbus.get_name_owner(service.try_into()?).await?;
        Ok(Binding {
            owner: owner.to_string(),
            path: path.into(),
            interface: interface.into(),
        })
    }
    async fn proxy<'a>(&'a self, binding: &Binding) -> Result<Proxy<'a>> {
        Ok(Proxy::new(
            &self.connection,
            binding.owner.clone(),
            binding.path.clone(),
            binding.interface.clone(),
        )
        .await?)
    }
    async fn properties(&self, binding: &Binding) -> Result<Properties> {
        let proxy = Proxy::new(
            &self.connection,
            binding.owner.clone(),
            binding.path.clone(),
            "org.freedesktop.DBus.Properties",
        )
        .await?;
        Ok(proxy.call("GetAll", &(binding.interface.as_str(),)).await?)
    }
    async fn activate_optional(&self) {
        async fn activate(connection: &Connection, service: &str) -> Result<()> {
            let dbus = Proxy::new(
                connection,
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
            )
            .await?;
            let owner: zbus::Result<String> = dbus.call("GetNameOwner", &(service,)).await;
            if owner.is_err() {
                let _: u32 = dbus.call("StartServiceByName", &(service, 0u32)).await?;
            }
            Ok(())
        }
        // Activate optional D-Bus services only when the user opens Power.
        // This neither enables systemd units nor starts a polling process.
        let _ = tokio::join!(
            tokio::time::timeout(Duration::from_secs(2), activate(&self.connection, UPOWER)),
            tokio::time::timeout(Duration::from_secs(2), activate(&self.connection, PROFILES))
        );
    }
}

fn text(properties: &Properties, name: &str, max: usize) -> String {
    properties
        .get(name)
        .and_then(|value| <&str>::try_from(value).ok())
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect()
}
fn flag(properties: &Properties, name: &str) -> bool {
    properties
        .get(name)
        .and_then(|value| bool::try_from(value).ok())
        .unwrap_or(false)
}
fn integer(properties: &Properties, name: &str) -> u64 {
    properties
        .get(name)
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or(0)
}

impl Backend for Native {
    fn snapshot(&self) -> BoxFuture<'_, Result<Hardware>> {
        Box::pin(async move {
            let (battery, profiles, session, backlights) = tokio::join!(
                tokio::time::timeout(Duration::from_secs(2), battery::read(self)),
                tokio::time::timeout(Duration::from_secs(2), profiles::read(self)),
                tokio::time::timeout(Duration::from_secs(3), session::read(self)),
                tokio::time::timeout(Duration::from_secs(2), self.backlights())
            );
            Ok(Hardware {
                battery: battery.unwrap_or_default(),
                profiles: profiles.unwrap_or_default(),
                session: session.unwrap_or_default(),
                backlights: backlights.ok().and_then(Result::ok).unwrap_or_default(),
            })
        })
    }
    fn backlights(&self) -> BoxFuture<'_, Result<Vec<Backlight>>> {
        Box::pin(async move {
            let permit = self.backlight_io.clone().acquire_owned().await?;
            let root = self.backlight_root.clone();
            tokio::task::spawn_blocking(move || {
                let _permit = permit;
                backlight::scan(&root)
            })
            .await?
        })
    }
    fn apply(&self, mutation: Mutation, observed: Hardware) -> BoxFuture<'_, Result<String>> {
        Box::pin(async move {
            match mutation {
                Mutation::Brightness { device, percent } => {
                    backlight::set(self, observed, &device, percent).await
                }
                Mutation::BrightnessRestore {
                    device,
                    expected,
                    value,
                } => backlight::restore(self, observed, &device, expected, value).await,
                Mutation::BrightnessDim {
                    device,
                    expected,
                    percent,
                } => {
                    observed.validate(&super::Request::Brightness {
                        device: device.clone(),
                        percent,
                    })?;
                    let light = observed
                        .backlights
                        .iter()
                        .find(|l| l.id == device)
                        .context("Backlight disappeared")?;
                    let value =
                        ((u64::from(light.max) * u64::from(percent) + 50) / 100).max(1) as u32;
                    backlight::restore(self, observed, &device, expected, value).await
                }
                Mutation::Profile(profile) => profiles::set(self, observed, profile).await,
                Mutation::Session { kind, target } => session::act(self, kind, target).await,
            }
        })
    }
}

async fn events(connection: &Connection) -> Result<SelectAll<MessageStream>> {
    let mut streams = SelectAll::new();
    for (service, path) in [
        (UPOWER, "/org/freedesktop/UPower"),
        (LOGIND, "/org/freedesktop/login1"),
        (PROFILES, "/org/freedesktop/UPower/PowerProfiles"),
        (LEGACY_PROFILES, "/net/hadess/PowerProfiles"),
    ] {
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .sender(service)?
            .path_namespace(path)?
            .build();
        streams.push(MessageStream::for_match_rule(rule, connection, Some(16)).await?);
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .sender("org.freedesktop.DBus")?
            .interface("org.freedesktop.DBus")?
            .member("NameOwnerChanged")?
            .arg(0, service)?
            .build();
        streams.push(MessageStream::for_match_rule(rule, connection, Some(8)).await?);
    }
    Ok(streams)
}

pub(super) async fn run(controller: Arc<Controller>) -> Result<()> {
    let native = Arc::new(Native::new(
        Connection::system().await?,
        PathBuf::from("/sys/class/backlight"),
        std::env::var("XDG_SESSION_ID")
            .ok()
            .filter(|id| !id.is_empty()),
        rustix::process::geteuid().as_raw(),
    ));
    observe(controller, native).await
}

pub(super) async fn observe(controller: Arc<Controller>, native: Arc<Native>) -> Result<()> {
    let mut events = events(&native.connection).await?;
    *controller.backend.lock().unwrap() = Some(native.clone());
    controller.refresh(native.as_ref()).await?;
    controller.state.send_modify(|state| {
        state.message.clear();
        state.error.clear();
    });
    let mut was_active = false;
    let mut sample_at = tokio::time::Instant::now();
    loop {
        controller.expire();
        let active = controller.active.load(Ordering::SeqCst)
            && controller.observers.load(Ordering::SeqCst) > 0;
        if active && !was_active {
            native.activate_optional().await;
            controller.refresh(native.as_ref()).await?;
            sample_at = tokio::time::Instant::now() + Duration::from_secs(2);
        }
        was_active = active;
        let expiry = controller
            .pending
            .lock()
            .unwrap()
            .as_ref()
            .map(|pending| pending.deadline);
        let deadline = if active {
            Some(expiry.map_or(sample_at, |expiry| expiry.min(sample_at)))
        } else {
            expiry
        };
        let timer = async {
            if let Some(deadline) = deadline {
                tokio::time::sleep_until(deadline).await;
            } else {
                std::future::pending::<()>().await;
            }
        };
        tokio::select! {
            event = events.next() => {
                event.context("Power bus stream ended")??;
                controller.refresh(native.as_ref()).await?;
            }
            _ = controller.wake.notified() => (),
            _ = timer => {
                controller.expire();
                if active && tokio::time::Instant::now() >= sample_at {
                    let lights = tokio::time::timeout(Duration::from_secs(2), native.backlights()).await;
                    if let Ok(Ok(lights)) = lights {
                        controller.state.send_if_modified(|state| {
                            if state.hardware.backlights == lights { return false; }
                            state.hardware.backlights = lights; true
                        });
                    }
                    sample_at = tokio::time::Instant::now() + Duration::from_secs(2);
                }
            }
        }
    }
}
