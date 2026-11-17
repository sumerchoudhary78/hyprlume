use futures_util::{SinkExt, StreamExt};
use hyprlume::{config::Config, protocol::Operation, server};
use serde_json::Value;
use std::{os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

struct Daemon {
    _directory: tempfile::TempDir,
    path: PathBuf,
    task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl Daemon {
    async fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("private/control.sock");
        let task = tokio::spawn(server::serve(path.clone(), Config::default(), true));
        tokio::time::timeout(Duration::from_secs(3), async {
            while !path.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        Self {
            _directory: dir,
            path,
            task,
        }
    }

    async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
        assert!(!self.path.exists());
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tokio::test]
async fn notification_topic_is_opt_in_and_overlay_switches_preserve_agent_state() {
    let daemon = Daemon::start().await;
    let mut observer = Framed::new(
        UnixStream::connect(&daemon.path).await.unwrap(),
        LinesCodec::new(),
    );
    observer
        .send(r#"{"id":1,"op":"notifications_subscribe"}"#)
        .await
        .unwrap();
    let initial: Value = serde_json::from_str(&observer.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(initial["id"], 1);
    let notifications = |request| Operation::Notifications { request };
    use hyprlume::services::notifications::Request as NotificationRequest;
    server::call(&daemon.path, notifications(NotificationRequest::Toggle))
        .await
        .unwrap();
    let event: Value = serde_json::from_str(&observer.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(event["event"], "notifications");
    assert_eq!(event["state"]["panel_open"], true);
    server::call(
        &daemon.path,
        notifications(NotificationRequest::Dnd { enabled: true }),
    )
    .await
    .unwrap();
    let event: Value = serde_json::from_str(&observer.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(event["state"]["dnd"], true);
    server::call(&daemon.path, Operation::Toggle).await.unwrap();
    let event: Value = serde_json::from_str(&observer.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(event["state"]["panel_open"], false);
    assert_eq!(event["state"]["dnd"], true);
    assert_eq!(
        server::call(&daemon.path, Operation::Status).await.unwrap()["panel_open"],
        true
    );
    // A notification-only subscriber never receives command-state messages.
    assert!(
        tokio::time::timeout(Duration::from_millis(50), observer.next())
            .await
            .is_err()
    );
    daemon.stop().await;
}

#[tokio::test]
async fn real_socket_pushes_updates_and_enforces_single_instance() {
    let daemon = Daemon::start().await;
    let path = &daemon.path;
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(server::serve(path.clone(), Config::default(), true)
        .await
        .is_err());
    assert!(server::call(path, Operation::Status).await.is_ok());
    let mut client = Framed::new(UnixStream::connect(&path).await.unwrap(), LinesCodec::new());
    client.send(r#"{"id":8,"op":"subscribe"}"#).await.unwrap();
    let response: Value = serde_json::from_str(&client.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(response["id"], 8);
    server::call(path, Operation::Toggle).await.unwrap();
    let event = tokio::time::timeout(Duration::from_secs(1), client.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let event: Value = serde_json::from_str(&event).unwrap();
    assert_eq!(event["event"], "state");
    assert_eq!(event["state"]["panel_open"], true);
    daemon.stop().await;
}

#[tokio::test]
async fn insecure_directory_and_regular_file_are_not_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("control.sock");
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(server::serve(path.clone(), Config::default(), true)
        .await
        .is_err());
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(&path, "keep me").unwrap();
    assert!(server::serve(path.clone(), Config::default(), true)
        .await
        .is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "keep me");
}

#[tokio::test]
async fn invalid_and_oversized_frames_do_not_stop_the_daemon() {
    let daemon = Daemon::start().await;
    let mut client = Framed::new(
        UnixStream::connect(&daemon.path).await.unwrap(),
        LinesCodec::new(),
    );
    client
        .send(r#"{"id":1,"op":"exec","command":"sh"}"#)
        .await
        .unwrap();
    let error: Value = serde_json::from_str(&client.next().await.unwrap().unwrap()).unwrap();
    assert!(error["id"].is_null());
    assert_eq!(error["error"], "invalid request");
    client.send(r#"{"id":2,"op":"status"}"#).await.unwrap();
    let reply: Value = serde_json::from_str(&client.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(reply["id"], 2);
    assert_eq!(reply["result"]["phase"], "ready");

    let mut oversized = UnixStream::connect(&daemon.path).await.unwrap();
    oversized
        .write_all(format!("{}\n", "x".repeat(server::MAX_FRAME + 1)).as_bytes())
        .await
        .unwrap();
    let mut byte = [0];
    let closed = tokio::time::timeout(Duration::from_secs(2), oversized.read(&mut byte))
        .await
        .unwrap();
    match closed {
        Ok(0) => {}
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
        other => panic!("oversized client remained open: {other:?}"),
    }
    assert!(server::call(&daemon.path, Operation::Status).await.is_ok());
    daemon.stop().await;
}

#[tokio::test]
async fn a_client_must_subscribe_before_receiving_state_events() {
    let daemon = Daemon::start().await;
    let mut client = Framed::new(
        UnixStream::connect(&daemon.path).await.unwrap(),
        LinesCodec::new(),
    );
    client.send(r#"{"id":1,"op":"status"}"#).await.unwrap();
    client.next().await.unwrap().unwrap();
    server::call(&daemon.path, Operation::Toggle).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(50), client.next())
            .await
            .is_err()
    );
    client.send(r#"{"id":2,"op":"status"}"#).await.unwrap();
    let reply: Value = serde_json::from_str(&client.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(reply["id"], 2);
    assert_eq!(reply["result"]["panel_open"], true);
    daemon.stop().await;
}

#[tokio::test]
async fn wireless_events_are_opt_in_and_both_topics_share_one_socket() {
    let daemon = Daemon::start().await;
    let mut agent = Framed::new(
        UnixStream::connect(&daemon.path).await.unwrap(),
        LinesCodec::new(),
    );
    agent.send(r#"{"id":1,"op":"subscribe"}"#).await.unwrap();
    agent.next().await.unwrap().unwrap();
    let mut wireless = Framed::new(
        UnixStream::connect(&daemon.path).await.unwrap(),
        LinesCodec::new(),
    );
    wireless
        .send(r#"{"id":2,"op":"connectivity_subscribe"}"#)
        .await
        .unwrap();
    let initial: Value = serde_json::from_str(&wireless.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(initial["id"], 2);
    assert_eq!(initial["result"]["panel_open"], false);

    let toggle = || Operation::Connectivity {
        request: hyprlume::services::connectivity::Request::Toggle,
    };
    server::call(&daemon.path, toggle()).await.unwrap();
    let event: Value = serde_json::from_str(&wireless.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(event["event"], "connectivity");
    assert_eq!(event["state"]["panel_open"], true);
    // Opening wireless hides the agent overlay, but legacy clients still get only agent state.
    let event: Value = serde_json::from_str(&agent.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(event["event"], "state");
    assert!(
        tokio::time::timeout(Duration::from_millis(50), agent.next())
            .await
            .is_err()
    );

    agent
        .send(r#"{"id":3,"op":"connectivity_subscribe"}"#)
        .await
        .unwrap();
    let reply: Value = serde_json::from_str(&agent.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(reply["id"], 3);
    assert_eq!(reply["result"]["panel_open"], true);
    server::call(&daemon.path, Operation::Toggle).await.unwrap();
    let mut events = Vec::new();
    for _ in 0..2 {
        let event: Value = serde_json::from_str(&agent.next().await.unwrap().unwrap()).unwrap();
        assert_eq!(event["state"]["panel_open"], event["event"] == "state");
        events.push(event["event"].as_str().unwrap().to_owned());
    }
    events.sort();
    assert_eq!(events, ["connectivity", "state"]);
    let event: Value = serde_json::from_str(&wireless.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(event["event"], "connectivity");
    assert_eq!(event["state"]["panel_open"], false);
    assert!(
        tokio::time::timeout(Duration::from_millis(50), wireless.next())
            .await
            .is_err()
    );
    daemon.stop().await;
}

#[tokio::test]
async fn wireless_validation_cannot_replace_an_agent_confirmation() {
    let daemon = Daemon::start().await;
    server::call(
        &daemon.path,
        Operation::Command {
            text: "workspace 2".into(),
            dry_run: true,
        },
    )
    .await
    .unwrap();
    let before = server::call(&daemon.path, Operation::Status).await.unwrap();
    assert!(before["confirmation"].is_number());
    server::call(
        &daemon.path,
        Operation::Connectivity {
            request: hyprlume::services::connectivity::Request::Toggle,
        },
    )
    .await
    .unwrap();
    let rejected = server::call(
        &daemon.path,
        Operation::Connectivity {
            request: hyprlume::services::connectivity::Request::WifiConnect {
                network: "stale-network".into(),
                password: None,
            },
        },
    )
    .await;
    assert!(rejected.is_err());
    let after = server::call(&daemon.path, Operation::Status).await.unwrap();
    assert_eq!(after["confirmation"], before["confirmation"]);
    assert_eq!(after["actions"], before["actions"]);
    assert_eq!(after["panel_open"], false);
    server::call(
        &daemon.path,
        Operation::Notifications {
            request: hyprlume::services::notifications::Request::Toggle,
        },
    )
    .await
    .unwrap();
    let after = server::call(&daemon.path, Operation::Status).await.unwrap();
    assert_eq!(after["confirmation"], before["confirmation"]);
    assert_eq!(after["actions"], before["actions"]);
    server::call(&daemon.path, Operation::Toggle).await.unwrap();
    assert_eq!(
        server::call(&daemon.path, Operation::NotificationsStatus)
            .await
            .unwrap()["panel_open"],
        false
    );
    daemon.stop().await;
}

#[tokio::test]
async fn command_shortcut_reveals_a_preview_arriving_behind_connections() {
    let daemon = Daemon::start().await;
    server::call(
        &daemon.path,
        Operation::Connectivity {
            request: hyprlume::services::connectivity::Request::Toggle,
        },
    )
    .await
    .unwrap();
    server::call(
        &daemon.path,
        Operation::Command {
            text: "workspace 2".into(),
            dry_run: true,
        },
    )
    .await
    .unwrap();
    let pending = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let state = server::call(&daemon.path, Operation::Status).await.unwrap();
            if state["confirmation"].is_number() {
                break state;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(pending["panel_open"], true);
    let wireless = server::call(&daemon.path, Operation::ConnectivityStatus)
        .await
        .unwrap();
    assert_eq!(wireless["panel_open"], true);
    let focused = server::call(&daemon.path, Operation::Toggle).await.unwrap();
    let wireless = server::call(&daemon.path, Operation::ConnectivityStatus)
        .await
        .unwrap();
    assert_eq!(focused["panel_open"], true);
    assert_eq!(focused["confirmation"], pending["confirmation"]);
    assert_eq!(wireless["panel_open"], false);
    daemon.stop().await;
}
