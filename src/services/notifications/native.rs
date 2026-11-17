//! freedesktop.org notification interface. Never replaces an existing owner.
use super::{
    types::{Incoming, Signal},
    Controller,
};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::Instant,
};
use zbus::{fdo, message::Header, object_server::SignalEmitter, zvariant::OwnedValue, Connection};

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";

struct Interface(Weak<Controller>);

impl Interface {
    fn controller(&self) -> fdo::Result<Arc<Controller>> {
        self.0
            .upgrade()
            .ok_or_else(|| fdo::Error::Failed("Notification server stopped".into()))
    }
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl Interface {
    fn get_capabilities(&self) -> Vec<&str> {
        vec!["actions", "body"]
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("Hyprlume", "Hyprlume", env!("CARGO_PKG_VERSION"), "1.3")
    }

    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<u32> {
        let controller = self.controller()?;
        if !controller.state().available
            && header
                .destination()
                .is_none_or(|destination| destination.as_str() != NAME)
        {
            return Err(fdo::Error::AccessDenied(
                "Hyprlume does not own the notification service".into(),
            ));
        }
        let owner = header
            .sender()
            .ok_or_else(|| fdo::Error::AccessDenied("Notification sender is missing".into()))?
            .as_str();
        let urgency = hints
            .get("urgency")
            .and_then(|value| u8::try_from(value).ok())
            .unwrap_or(1);
        let flag = |name| {
            hints
                .get(name)
                .and_then(|value| bool::try_from(value).ok())
                .unwrap_or(false)
        };
        let input = Incoming {
            app_name,
            replaces_id,
            summary,
            body,
            actions,
            urgency,
            timeout_ms: expire_timeout,
            transient: flag("transient"),
            resident: flag("resident"),
        };
        let (id, signals) = controller
            .change(|store| store.receive(owner, input, Instant::now()))
            .map_err(|error| fdo::Error::InvalidArgs(error.to_string()))?;
        controller
            .emit(signals)
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        Ok(id)
    }

    async fn close_notification(
        &self,
        id: u32,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        let controller = self.controller()?;
        let owner = header
            .sender()
            .ok_or_else(|| fdo::Error::AccessDenied("Notification sender is missing".into()))?
            .as_str();
        let signals = controller
            .change(|store| store.close_from_client(owner, id))
            .map_err(|error| fdo::Error::InvalidArgs(error.to_string()))?;
        controller
            .emit(signals)
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    #[zbus(signal)]
    async fn notification_closed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;
    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}

struct Lease(Arc<Controller>);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.bus.lock().unwrap().take();
        self.0.unavailable("Notification service stopped");
    }
}

pub(super) async fn run(controller: Arc<Controller>) -> Result<()> {
    serve(controller, zbus::connection::Builder::session()?).await
}

pub(super) async fn serve(
    controller: Arc<Controller>,
    builder: zbus::connection::Builder<'_>,
) -> Result<()> {
    let connection = builder
        .max_queued(32)
        .serve_at(PATH, Interface(Arc::downgrade(&controller)))?
        .build()
        .await?;
    // Subscribe before trying the name so release/reacquisition cannot miss a race.
    let bus = fdo::DBusProxy::new(&connection).await?;
    let mut owners = bus
        .receive_name_owner_changed_with_args(&[(0, NAME)])
        .await?;
    loop {
        match connection
            .request_name_with_flags(NAME, fdo::RequestNameFlags::DoNotQueue.into())
            .await
        {
            Ok(_) => break,
            Err(zbus::Error::NameTaken) => controller.unavailable(
                "Another notification daemon owns this session. Hyprlume has not replaced it.",
            ),
            Err(error) => return Err(error.into()),
        }
        loop {
            let signal = owners
                .next()
                .await
                .context("Notification ownership stream closed")?;
            if signal.args()?.new_owner().is_none() {
                break;
            }
        }
    }
    *controller.bus.lock().unwrap() = Some(connection.clone());
    let _lease = Lease(controller.clone());
    controller.change(|store| {
        store.state.available = true;
        store.state.error.clear();
        Ok(())
    })?;
    let disconnected = async {
        let mut messages = zbus::MessageStream::from(&connection);
        while let Some(Ok(_)) = messages.next().await {}
    };
    tokio::select! { _ = controller.expire_loop() => {}, _ = disconnected => {} }
    Ok(())
}

pub(super) async fn emit(connection: &Connection, signals: Vec<Signal>) -> Result<()> {
    for signal in signals {
        match signal {
            Signal::Closed { owner, id, reason } => {
                connection
                    .emit_signal(
                        Some(owner.as_str()),
                        PATH,
                        NAME,
                        "NotificationClosed",
                        &(id, reason),
                    )
                    .await?
            }
            Signal::Action { owner, id, key } => {
                connection
                    .emit_signal(
                        Some(owner.as_str()),
                        PATH,
                        NAME,
                        "ActionInvoked",
                        &(id, key),
                    )
                    .await?
            }
        }
    }
    Ok(())
}
