use super::{
    feed::{self, Palette},
    APPEARANCE_BACKEND, PATH,
};
use crate::config::{Accent, Appearance};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::watch;
use zbus::{
    connection::Builder,
    fdo::{self, DBusProxy},
    object_server::SignalEmitter,
    zvariant::{OwnedValue, Value},
};

const NAMESPACE: &str = "org.freedesktop.appearance";
const KEYS: [&str; 4] = ["color-scheme", "accent-color", "reduced-motion", "contrast"];
type Settings = HashMap<String, HashMap<String, OwnedValue>>;

struct Interface(watch::Receiver<Palette>);

#[zbus::interface(name = "org.freedesktop.impl.portal.Settings")]
impl Interface {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }

    fn read_all(&self, namespaces: Vec<String>) -> Settings {
        if !namespaces.is_empty() && !namespaces.iter().any(|pattern| matches_namespace(pattern)) {
            return Settings::new();
        }
        let palette = *self.0.borrow();
        let values = KEYS
            .into_iter()
            .map(|key| {
                (
                    key.into(),
                    value(palette, key).expect("defined appearance key"),
                )
            })
            .collect();
        Settings::from([(NAMESPACE.into(), values)])
    }

    fn read(&self, namespace: &str, key: &str) -> fdo::Result<OwnedValue> {
        if namespace == NAMESPACE {
            if let Some(value) = value(*self.0.borrow(), key) {
                return Ok(value);
            }
        }
        Err(fdo::Error::InvalidArgs("Unknown appearance setting".into()))
    }

    #[zbus(signal)]
    async fn setting_changed(
        emitter: &SignalEmitter<'_>,
        namespace: &str,
        key: &str,
        value: OwnedValue,
    ) -> zbus::Result<()>;
}

fn matches_namespace(pattern: &str) -> bool {
    pattern.is_empty()
        || pattern == NAMESPACE
        || pattern.strip_suffix(".*").is_some_and(|prefix| {
            NAMESPACE
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.'))
        })
}

fn value(palette: Palette, key: &str) -> Option<OwnedValue> {
    match key {
        "color-scheme" => Some(OwnedValue::from(palette.app_color.portal_value())),
        "contrast" => Some(OwnedValue::from(u32::from(palette.high_contrast))),
        "reduced-motion" => Some(OwnedValue::from(u32::from(palette.reduced_effects))),
        "accent-color" => {
            // sRGB values from the shell's accent tokens in Theme.qml.
            let [r, g, b] = match palette.accent {
                Accent::Cyan => [92, 225, 255],
                Accent::Violet => [180, 151, 255],
                Accent::Mint => [104, 240, 177],
            }
            .map(|channel| f64::from(channel) / 255.0);
            OwnedValue::try_from(Value::new((r, g, b))).ok()
        }
        _ => None,
    }
}

pub(super) async fn serve(initial: Appearance, socket: Option<PathBuf>) -> Result<()> {
    let mut palette = Palette {
        accent: initial.accent,
        reduced_effects: initial.reduced_effects,
        high_contrast: initial.high_contrast,
        app_color: initial.app_color,
    };
    let stream = if let Some(path) = &socket {
        if let Ok((stream, current)) = feed::connect(path).await {
            palette = current;
            Some(stream)
        } else {
            None
        }
    } else {
        None
    };
    let (updates, changes) = watch::channel(palette);
    let connection = Builder::session()?
        .name(APPEARANCE_BACKEND)?
        .serve_at(PATH, Interface(changes.clone()))?
        .build()
        .await
        .context("Register Hyprlume's appearance portal on the session bus")?;
    let bus = DBusProxy::new(&connection).await?;
    let mut ownership = bus
        .receive_name_owner_changed_with_args(&[(0, APPEARANCE_BACKEND)])
        .await?;
    let interface = connection
        .object_server()
        .interface::<_, Interface>(PATH)
        .await?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        _ = terminate.recv() => Ok(()),
        _ = tokio::signal::ctrl_c() => Ok(()),
        _ = ownership.next() => Ok(()),
        _ = async {
            if let Some(path) = &socket {
                feed::follow(path, stream, updates).await;
            } else {
                std::future::pending::<()>().await;
            }
        } => Ok(()),
        result = publish(changes, palette, interface.signal_emitter()) => result,
    }
}

async fn publish(
    mut changes: watch::Receiver<Palette>,
    mut previous: Palette,
    emitter: &SignalEmitter<'_>,
) -> Result<()> {
    while changes.changed().await.is_ok() {
        let current = *changes.borrow_and_update();
        for key in KEYS {
            let next = value(current, key).expect("defined appearance key");
            if Some(&next) != value(previous, key).as_ref() {
                Interface::setting_changed(emitter, NAMESPACE, key, next).await?;
            }
        }
        previous = current;
    }
    Ok(())
}
