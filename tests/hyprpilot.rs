use hyprlume::{config::Config, desktop::Desktop, planner::Action};
use hyprpilot_core::{Connection, Instance};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
};

async fn compositor(
    responses: Vec<(&str, &str)>,
) -> (tempfile::TempDir, Desktop, tokio::task::JoinHandle<()>) {
    let dir = tempfile::tempdir().unwrap();
    let instance = Instance {
        signature: "isolated-test".into(),
        runtime_dir: dir.path().to_owned(),
    };
    std::fs::create_dir_all(instance.control_socket().parent().unwrap()).unwrap();
    let listener = UnixListener::bind(instance.control_socket()).unwrap();
    let responses: Vec<_> = responses
        .into_iter()
        .map(|(q, a)| (q.to_owned(), a.to_owned()))
        .collect();
    let server = tokio::spawn(async move {
        for (expected, response) in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = String::new();
            stream.read_to_string(&mut request).await.unwrap();
            assert_eq!(request, expected);
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    (
        dir,
        Desktop {
            connection: Some(Connection::with_instance(instance)),
            demo: false,
        },
        server,
    )
}

#[tokio::test]
async fn workspace_dispatch_uses_hyprpilot_and_checks_the_result() {
    let (_dir, desktop, server) = compositor(vec![
        ("dispatch workspace 2", "ok"),
        (
            "j/activeworkspace",
            r#"{"id":2,"name":"2","monitor":"TEST","windows":0}"#,
        ),
    ])
    .await;
    assert_eq!(
        desktop
            .execute(&Action::Workspace { id: 2 }, &Config::default(), None)
            .await
            .unwrap(),
        "Workspace 2 confirmed"
    );
    server.await.unwrap();
}

#[tokio::test]
async fn an_acknowledgment_without_the_requested_state_is_not_success() {
    let (_dir, desktop, server) = compositor(vec![
        ("dispatch workspace 2", "ok"),
        (
            "j/activeworkspace",
            r#"{"id":1,"name":"1","monitor":"TEST","windows":0}"#,
        ),
    ])
    .await;
    assert!(desktop
        .execute(&Action::Workspace { id: 2 }, &Config::default(), None)
        .await
        .unwrap_err()
        .to_string()
        .contains("did not confirm"));
    server.await.unwrap();
}

#[tokio::test]
async fn a_missing_window_never_dispatches_a_guess() {
    let (_dir, desktop, server) = compositor(vec![("j/clients", "[]")]).await;
    assert!(desktop
        .execute(
            &Action::Focus {
                app: "firefox".into()
            },
            &Config::default(),
            None
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("found 0"));
    server.await.unwrap();
}

#[tokio::test]
async fn moving_a_bound_window_never_retargets_the_new_active_window() {
    let before = serde_json::json!([
        {"address":"0xa","mapped":true,"at":[0,0],"size":[800,600],"workspace":{"id":1,"name":"1"},"floating":false,"monitor":0,"class":"firefox","title":"A","pid":10,"xwayland":false},
        {"address":"0xb","mapped":true,"at":[0,0],"size":[800,600],"workspace":{"id":2,"name":"2"},"floating":false,"monitor":0,"class":"kitty","title":"B","pid":11,"xwayland":false}
    ]);
    let mut after = before.clone();
    after[0]["workspace"]["id"] = 3.into();
    let (before, after) = (before.to_string(), after.to_string());
    let (_dir, desktop, server) = compositor(vec![
        ("j/clients", &before),
        ("dispatch movetoworkspace 3,address:0xa", "ok"),
        ("j/clients", &after),
    ])
    .await;
    assert!(desktop
        .execute(
            &Action::MoveWindow { id: 3 },
            &Config::default(),
            Some("0xa")
        )
        .await
        .unwrap()
        .contains("firefox moved"));
    server.await.unwrap();
}
