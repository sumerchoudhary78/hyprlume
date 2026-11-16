use futures_util::{SinkExt, StreamExt};
use hyprlume::{config::Config, protocol::Operation, server};
use serde_json::Value;
use std::{os::unix::fs::PermissionsExt, time::Duration};
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::test]
async fn real_socket_pushes_updates_and_enforces_single_instance() {
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
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(server::serve(path.clone(), Config::default(), true)
        .await
        .is_err());
    assert!(server::call(&path, Operation::Status).await.is_ok());
    let mut client = Framed::new(UnixStream::connect(&path).await.unwrap(), LinesCodec::new());
    client.send(r#"{"id":8,"op":"subscribe"}"#).await.unwrap();
    let response: Value = serde_json::from_str(&client.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(response["id"], 8);
    server::call(&path, Operation::Toggle).await.unwrap();
    let event = tokio::time::timeout(Duration::from_secs(1), client.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let event: Value = serde_json::from_str(&event).unwrap();
    assert_eq!(event["event"], "state");
    assert_eq!(event["state"]["panel_open"], true);
    task.abort();
    let _ = task.await;
    assert!(!path.exists());
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
