use hyprlume::{
    config::Config,
    planner::{direct, Action, Plan},
    protocol::{Operation, Request},
    server::App,
};
use serde_json::json;
use std::time::Duration;

fn config() -> Config {
    let mut config = Config::default();
    config.apps.insert("browser".into(), vec!["firefox".into()]);
    config
}

#[test]
fn natural_commands_resolve_to_typed_actions() {
    for (text, expected) in [
        (
            "Please switch to workspace two.",
            Action::Workspace { id: 2 },
        ),
        (
            "move this window to workspace 4",
            Action::MoveWindow { id: 4 },
        ),
        (
            "focus firefox",
            Action::Focus {
                app: "firefox".into(),
            },
        ),
        (
            "open browser",
            Action::Launch {
                app: "browser".into(),
            },
        ),
        ("set volume to 30 percent", Action::Volume { percent: 30 }),
        (
            "set the volume to thirty percent",
            Action::Volume { percent: 30 },
        ),
        ("volume twenty-five", Action::Volume { percent: 25 }),
        ("go to work space too", Action::Workspace { id: 2 }),
        ("unmute audio", Action::Mute { muted: false }),
    ] {
        assert_eq!(
            direct(text, &config()).unwrap().unwrap().actions,
            [expected]
        );
    }
}

#[test]
fn unknown_requests_never_turn_into_shell_commands() {
    for text in [
        "open browser; touch /tmp/pwned",
        "rm -rf /",
        "open $(whoami)",
        "type sudo reboot",
        "open browser and move it to workspace 2",
    ] {
        assert!(direct(text, &config()).unwrap().is_none(), "{text}");
    }
    assert!(direct("workspace 2\nexec bad", &config()).is_err());
    assert!(direct(&"x".repeat(2049), &config()).is_err());
}

#[test]
fn limits_apply_to_direct_and_model_plans() {
    for text in [
        "workspace 0",
        "workspace -1",
        "workspace 11",
        "volume 101",
        "volume -1",
        "workspace 2; exec bad",
        "focus firefox;exec bad",
    ] {
        assert!(direct(text, &config()).is_err());
    }
    assert!(Plan {
        source: "llm".into(),
        actions: vec![Action::Launch {
            app: "unconfigured".into()
        }]
    }
    .validate(&config())
    .is_err());
    assert!(Plan {
        source: "llm".into(),
        actions: vec![Action::Mute { muted: true }; 5]
    }
    .validate(&config())
    .is_err());
    assert!(serde_json::from_value::<Action>(
        json!({"type":"launch","app":"browser","command":"evil"})
    )
    .is_err());
    assert!(serde_json::from_value::<Action>(json!({"type":"exec","command":"evil"})).is_err());
}

#[test]
fn ipc_omitted_execute_flag_means_preview() {
    let request: Request =
        serde_json::from_value(json!({"id":1,"op":"command","text":"workspace 2"})).unwrap();
    assert!(matches!(
        request.operation,
        Operation::Command { dry_run: true, .. }
    ));
}

#[test]
fn config_keeps_inference_local_and_validates_launchers() {
    let mut config = config();
    for endpoint in [
        "https://example.org/",
        "http://localhost:8080",
        "http://127.0.0.1.evil.test",
        "file:///etc/passwd",
        "http://user@127.0.0.1/",
    ] {
        config.llm.endpoint = Some(endpoint.into());
        assert!(config.validate().is_err(), "{endpoint}");
    }
    config.llm.endpoint = Some("http://127.0.0.1:8080/v1/chat/completions".into());
    assert!(config.validate().is_ok());
    config.apps.insert("broken".into(), vec![]);
    assert!(config.validate().is_err());
}

async fn wait(app: &App, phase: &str) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while app.state().phase != phase {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn preview_confirm_is_single_use_and_default_mic_is_off() {
    let app = App::new(config(), true);
    assert_eq!(app.state().voice, hyprlume::protocol::VoiceMode::Off);
    app.handle(Operation::Command {
        text: "workspace 2".into(),
        dry_run: true,
    })
    .await
    .unwrap();
    wait(&app, "review").await;
    assert!(app.state().receipts.is_empty());
    let token = app.state().confirmation.unwrap();
    assert!(app
        .handle(Operation::Confirm { token: token + 1 })
        .await
        .is_err());
    app.handle(Operation::Confirm { token }).await.unwrap();
    wait(&app, "done").await;
    assert!(app.state().receipts[0].contains("Demo only"));
    assert!(app.handle(Operation::Confirm { token }).await.is_err());
}

#[tokio::test]
async fn dismissal_invalidates_preview_and_direct_requests_work() {
    let app = App::new(config(), true);
    app.handle(Operation::Command {
        text: "volume 30".into(),
        dry_run: true,
    })
    .await
    .unwrap();
    wait(&app, "review").await;
    let token = app.state().confirmation.unwrap();
    app.handle(Operation::Cancel).await.unwrap();
    assert!(app.handle(Operation::Confirm { token }).await.is_err());
    app.handle(Operation::Command {
        text: "mute".into(),
        dry_run: false,
    })
    .await
    .unwrap();
    wait(&app, "done").await;
    assert_eq!(app.state().receipts.len(), 1);
}

#[tokio::test]
async fn a_new_request_cannot_replace_a_pending_confirmation() {
    let app = App::new(config(), true);
    app.handle(Operation::Command {
        text: "workspace 2".into(),
        dry_run: true,
    })
    .await
    .unwrap();
    wait(&app, "review").await;
    let token = app.state().confirmation;
    assert!(app
        .handle(Operation::Command {
            text: "workspace 3".into(),
            dry_run: false
        })
        .await
        .is_err());
    assert_eq!(app.state().confirmation, token);
    assert!(app.state().receipts.is_empty());
}

#[tokio::test]
async fn turning_the_microphone_off_preserves_a_pending_preview() {
    let app = App::new(config(), true);
    app.handle(Operation::Command {
        text: "workspace 2".into(),
        dry_run: true,
    })
    .await
    .unwrap();
    wait(&app, "review").await;
    let token = app.state().confirmation;
    let _ = app
        .handle(Operation::Voice {
            mode: hyprlume::protocol::VoiceMode::Off,
        })
        .await;
    assert_eq!(app.state().confirmation, token);
    assert_eq!(app.state().phase, "review");
}
