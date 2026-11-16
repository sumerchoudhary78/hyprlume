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
        ("j/status", "unknown request"),
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
        ("j/status", r#"{"configProvider":"hyprlang"}"#),
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
        ("j/status", "unknown request"),
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

#[tokio::test]
async fn lua_workspace_dispatch_is_checked_against_desktop_state() {
    let (_dir, desktop, server) = compositor(vec![
        ("j/status", r#"{"configProvider":"lua"}"#),
        ("dispatch hl.dsp.focus({ workspace = 2 })", "ok"),
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

fn sample_window(address: &str, workspace: i32) -> serde_json::Value {
    serde_json::json!({"address":address,"mapped":true,"at":[0,0],"size":[800,600],"workspace":{"id":workspace,"name":workspace.to_string()},"floating":false,"monitor":0,"class":"firefox","title":"Test","pid":10,"xwayland":false})
}

#[tokio::test]
async fn lua_move_keeps_the_original_window_address() {
    let before = serde_json::json!([sample_window("0xa", 1), sample_window("0xb", 2)]).to_string();
    let after = serde_json::json!([sample_window("0xa", 3), sample_window("0xb", 2)]).to_string();
    let (_dir,desktop,server)=compositor(vec![
        ("j/clients",&before),
        ("j/status",r#"{"configProvider":"lua"}"#),
        ("dispatch hl.dsp.window.move({ workspace = 3, window = \"address:0xa\", follow = true })","ok"),
        ("j/clients",&after),
    ]).await;
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

#[tokio::test]
async fn lua_focus_uses_a_unique_bound_address() {
    let window = sample_window("0xa", 1).to_string();
    let clients = format!("[{window}]");
    let (_dir, desktop, server) = compositor(vec![
        ("j/clients", &clients),
        ("j/status", r#"{"configProvider":"lua"}"#),
        ("dispatch hl.dsp.focus({ window = \"address:0xa\" })", "ok"),
        ("j/activewindow", &window),
    ])
    .await;
    assert!(desktop
        .execute(
            &Action::Focus {
                app: "firefox".into()
            },
            &Config::default(),
            None
        )
        .await
        .unwrap()
        .contains("focused"));
    server.await.unwrap();
}

#[tokio::test]
async fn malformed_config_status_never_guesses_a_dispatcher() {
    let (_dir, desktop, server) = compositor(vec![("j/status", r#"{"configProvider":42}"#)]).await;
    assert!(desktop
        .execute(&Action::Workspace { id: 2 }, &Config::default(), None)
        .await
        .unwrap_err()
        .to_string()
        .contains("invalid config status"));
    server.await.unwrap();
}

#[tokio::test]
async fn an_invalid_address_never_becomes_lua_code() {
    let address = "0xa\" }); injected() --";
    let clients = serde_json::json!([sample_window(address, 1)]).to_string();
    let (_dir, desktop, server) = compositor(vec![
        ("j/clients", &clients),
        ("j/status", r#"{"configProvider":"lua"}"#),
    ])
    .await;
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
        .contains("invalid window address"));
    server.await.unwrap();
}
