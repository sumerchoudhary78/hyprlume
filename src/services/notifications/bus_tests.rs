//! Real D-Bus tests on an owned private bus; never connect to the user's session.
use super::{native, Controller, Request};
use futures_util::StreamExt;
use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use zbus::{fdo, zvariant::OwnedValue, Connection, Proxy};

struct Bus {
    address: String,
    _child: Child,
}
impl Bus {
    async fn new() -> Self {
        let mut child = Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("install dbus-daemon for isolated notification tests");
        let mut address = String::new();
        tokio::time::timeout(
            Duration::from_secs(3),
            BufReader::new(child.stdout.take().unwrap()).read_line(&mut address),
        )
        .await
        .unwrap()
        .unwrap();
        Self {
            address: address.trim().into(),
            _child: child,
        }
    }
    async fn connect(&self) -> Connection {
        zbus::connection::Builder::address(self.address.as_str())
            .unwrap()
            .build()
            .await
            .unwrap()
    }
    fn serve(&self, controller: Arc<Controller>) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        let address = self.address.clone();
        tokio::spawn(async move {
            native::serve(
                controller,
                zbus::connection::Builder::address(address.as_str())?,
            )
            .await
        })
    }
}

async fn wait(controller: &Controller, available: bool) {
    tokio::time::timeout(Duration::from_secs(3), async {
        while controller.state().available != available
            || !available && controller.state().error.is_empty()
        {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_methods_emit_owner_bound_action_and_close_signals() {
    let bus = Bus::new().await;
    let controller = Controller::new(false);
    let server = bus.serve(controller.clone());
    wait(&controller, true).await;
    let client = bus.connect().await;
    let rival = bus.connect().await;
    let proxy = Proxy::new(
        &client,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await
    .unwrap();
    let other = Proxy::new(
        &rival,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await
    .unwrap();
    let capabilities: Vec<String> = proxy.call("GetCapabilities", &()).await.unwrap();
    assert_eq!(capabilities, ["actions", "body"]);
    let information: (String, String, String, String) =
        proxy.call("GetServerInformation", &()).await.unwrap();
    assert_eq!(information.0, "Hyprlume");
    let mut actions = proxy.receive_signal("ActionInvoked").await.unwrap();
    let mut closes = proxy.receive_signal("NotificationClosed").await.unwrap();
    let mut foreign_closes = other.receive_signal("NotificationClosed").await.unwrap();
    let hints = HashMap::from([("urgency", OwnedValue::from(2u8))]);
    let id: u32 = proxy
        .call(
            "Notify",
            &(
                "Fixture app",
                0u32,
                "",
                "A <b>literal</b> title",
                "Private fixture body",
                vec!["open", "Open"],
                hints,
                0i32,
            ),
        )
        .await
        .unwrap();
    assert!(id > 0);
    let snapshot = controller.state();
    assert_eq!(snapshot.items[0].summary, "A <b>literal</b> title");
    assert_eq!(snapshot.items[0].urgency, 2);
    let rejected: zbus::Result<()> = other.call("CloseNotification", &(id,)).await;
    assert!(rejected.is_err());
    controller
        .handle(Request::Invoke {
            id,
            revision: snapshot.items[0].revision,
            key: "open".into(),
        })
        .await
        .unwrap();
    let action = tokio::time::timeout(Duration::from_secs(2), actions.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        action.body().deserialize::<(u32, String)>().unwrap(),
        (id, "open".into())
    );
    let closed = tokio::time::timeout(Duration::from_secs(2), closes.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(closed.body().deserialize::<(u32, u32)>().unwrap(), (id, 2));
    assert!(
        tokio::time::timeout(Duration::from_millis(50), foreign_closes.next())
            .await
            .is_err()
    );
    server.abort();
    let _ = server.await;
    assert!(!controller.state().available);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_replaceable_existing_server_is_never_displaced_and_release_is_observed() {
    let bus = Bus::new().await;
    let existing = bus.connect().await;
    existing
        .request_name_with_flags(
            "org.freedesktop.Notifications",
            fdo::RequestNameFlags::AllowReplacement | fdo::RequestNameFlags::DoNotQueue,
        )
        .await
        .unwrap();
    let controller = Controller::new(false);
    let server = bus.serve(controller.clone());
    wait(&controller, false).await;
    assert!(controller
        .state()
        .error
        .contains("Another notification daemon"));
    let proxy = fdo::DBusProxy::new(&existing).await.unwrap();
    let owner = proxy
        .get_name_owner("org.freedesktop.Notifications".try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(owner.as_str(), existing.unique_name().unwrap().as_str());
    // The pre-registered object must not accept notices addressed to its unique
    // connection while another server still owns the public service name.
    let unowned = proxy
        .list_names()
        .await
        .unwrap()
        .into_iter()
        .find(|name| {
            name.as_str().starts_with(':')
                && name.as_str() != existing.unique_name().unwrap().as_str()
        })
        .unwrap();
    let hidden = Proxy::new(
        &existing,
        unowned.as_str(),
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await
    .unwrap();
    let rejected: zbus::Result<u32> = hidden
        .call(
            "Notify",
            &(
                "Fixture",
                0u32,
                "",
                "Hidden target",
                "",
                Vec::<String>::new(),
                HashMap::<String, OwnedValue>::new(),
                0i32,
            ),
        )
        .await;
    assert!(rejected.is_err());
    assert!(controller.state().items.is_empty());
    existing
        .release_name("org.freedesktop.Notifications")
        .await
        .unwrap();
    wait(&controller, true).await;
    assert!(controller.state().error.is_empty());
    server.abort();
    let _ = server.await;
}
