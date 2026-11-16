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
