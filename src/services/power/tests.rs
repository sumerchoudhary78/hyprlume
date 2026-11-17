use super::types::{Binding, SessionTarget};
use super::*;
use std::sync::atomic::AtomicUsize;

fn hardware() -> Hardware {
    Hardware {
        backlights: vec![Backlight {
            id: "screen:1:2:1000".into(),
            name: "screen".into(),
            percent: 70,
            value: 700,
            max: 1000,
        }],
        profiles: Profiles {
            available: true,
            active: Some(Profile::Balanced),
            choices: vec![Profile::PowerSaver, Profile::Balanced],
            binding: Some(Binding {
                owner: ":1.55".into(),
                path: "/profiles".into(),
                interface: "profiles".into(),
            }),
            ..Profiles::default()
        },
        session: Session {
            available: true,
            id: "7".into(),
            suspend: "yes".into(),
            logout: "yes".into(),
            reboot: "challenge".into(),
            power_off: "inhibited".into(),
            error: String::new(),
            target: Some(SessionTarget {
                binding: Binding {
                    owner: ":1.56".into(),
                    path: "/session7".into(),
                    interface: "session".into(),
                },
                id: "7".into(),
                uid: 1000,
                timestamp: 1234,
            }),
        },
        ..Hardware::default()
    }
}
struct Fake {
    hardware: Mutex<Hardware>,
    calls: Mutex<Vec<Mutation>>,
    snapshot_gate: Mutex<Option<Arc<Notify>>>,
    apply_gate: Mutex<Option<Arc<Notify>>>,
    snapshots: AtomicUsize,
    fail: Mutex<Option<String>>,
}
impl Backend for Fake {
    fn snapshot(&self) -> BoxFuture<'_, Result<Hardware>> {
        Box::pin(async {
            self.snapshots.fetch_add(1, Ordering::SeqCst);
            let gate = self.snapshot_gate.lock().unwrap().clone();
            if let Some(gate) = gate {
                gate.notified().await;
            }
            Ok(self.hardware.lock().unwrap().clone())
        })
    }
    fn backlights(&self) -> BoxFuture<'_, Result<Vec<Backlight>>> {
        Box::pin(async { Ok(self.hardware.lock().unwrap().backlights.clone()) })
    }
    fn apply(&self, mutation: Mutation, _: Hardware) -> BoxFuture<'_, Result<String>> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(mutation);
            let gate = self.apply_gate.lock().unwrap().clone();
            if let Some(gate) = gate {
                gate.notified().await;
            }
            if let Some(error) = self.fail.lock().unwrap().as_ref() {
                anyhow::bail!("{error}");
            }
            Ok("Fixture accepted the request".into())
        })
    }
}
async fn fixture() -> (Arc<Controller>, Arc<Fake>) {
    let controller = Controller::new(true);
    let backend = Arc::new(Fake {
        hardware: Mutex::new(hardware()),
        calls: Mutex::new(Vec::new()),
        snapshot_gate: Mutex::new(None),
        apply_gate: Mutex::new(None),
        snapshots: AtomicUsize::new(0),
        fail: Mutex::new(None),
    });
    *controller.backend.lock().unwrap() = Some(backend.clone());
    controller.refresh(backend.as_ref()).await.unwrap();
    controller.set_active(true);
    (controller, backend)
}
async fn settle(controller: &Controller) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while controller.busy.available_permits() == 0 {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .unwrap();
}
async fn prepare(controller: &Arc<Controller>) -> Prompt {
    controller
        .handle(Request::Prepare {
            kind: SessionAction::Suspend,
        })
        .await
        .unwrap();
    settle(controller).await;
    controller.state().prompt.unwrap()
}

#[test]
fn strict_requests_and_advertised_capabilities_reject_unknown_or_stale_targets() {
    for input in [
        r#"{"action":"prepare","kind":"hibernate"}"#,
        r#"{"action":"profile","profile":"custom-command"}"#,
        r#"{"action":"confirm","token":1,"command":"shutdown"}"#,
        r#"{"action":"brightness","device":"screen","percent":256}"#,
    ] {
        assert!(serde_json::from_str::<Request>(input).is_err());
    }
    let hardware = hardware();
    assert!(hardware
        .validate(&Request::Brightness {
            device: "../../brightness".into(),
            percent: 50
        })
        .is_err());
    assert!(hardware
        .validate(&Request::Brightness {
            device: hardware.backlights[0].id.clone(),
            percent: 0
        })
        .is_err());
    assert!(hardware
        .validate(&Request::Profile {
            profile: Profile::Performance
        })
        .is_err());
    assert!(hardware
        .validate(&Request::Prepare {
            kind: SessionAction::PowerOff
        })
        .is_err());
    assert!(hardware
        .validate(&Request::Prepare {
            kind: SessionAction::Reboot
        })
        .is_ok());
    let public = serde_json::to_string(&hardware).unwrap();
    for hidden in [
        ":1.55",
        ":1.56",
        "/session7",
        "/profiles",
        "timestamp",
        "binding",
        "target",
    ] {
        assert!(!public.contains(hidden));
    }
}

#[tokio::test]
async fn a_session_action_needs_a_current_single_use_confirmation() {
    let (controller, backend) = fixture().await;
    let prompt = prepare(&controller).await;
    assert!(backend.calls.lock().unwrap().is_empty());
    assert_eq!(prompt.session_id, "7");
    assert!(controller
        .handle(Request::Confirm {
            token: prompt.token + 1
        })
        .await
        .is_err());
    assert!(controller
        .handle(Request::Cancel {
            token: prompt.token + 1
        })
        .await
        .is_err());
    assert_eq!(controller.state().prompt.unwrap().token, prompt.token);
    controller
        .handle(Request::Confirm {
            token: prompt.token,
        })
        .await
        .unwrap();
    settle(&controller).await;
    {
        let calls = backend.calls.lock().unwrap();
        assert!(
            matches!(&calls[0], Mutation::Session { kind: SessionAction::Suspend, target } if target.id == "7")
        );
        assert_eq!(calls.len(), 1);
    }
    assert!(controller
        .handle(Request::Confirm {
            token: prompt.token
        })
        .await
        .is_err());
}

#[tokio::test]
async fn expiry_cancellation_and_panel_close_invalidate_confirmation() {
    let (controller, backend) = fixture().await;
    let expired = prepare(&controller).await;
    controller
        .pending
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .deadline = tokio::time::Instant::now();
    controller.expire();
    assert!(controller.state().prompt.is_none());
    assert!(controller
        .handle(Request::Confirm {
            token: expired.token
        })
        .await
        .is_err());
    let cancelled = prepare(&controller).await;
    controller
        .handle(Request::Cancel {
            token: cancelled.token,
        })
        .await
        .unwrap();
    assert!(controller
        .handle(Request::Confirm {
            token: cancelled.token
        })
        .await
        .is_err());
    let closed = prepare(&controller).await;
    controller.set_active(false);
    controller.set_active(true);
    assert!(controller
        .handle(Request::Confirm {
            token: closed.token
        })
        .await
        .is_err());
    assert!(backend.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_replaced_graphical_session_cannot_inherit_a_confirmation() {
    let (controller, backend) = fixture().await;
    let prompt = prepare(&controller).await;
    backend
        .hardware
        .lock()
        .unwrap()
        .session
        .target
        .as_mut()
        .unwrap()
        .timestamp += 1;
    controller
        .handle(Request::Confirm {
            token: prompt.token,
        })
        .await
        .unwrap();
    settle(&controller).await;
    assert!(backend.calls.lock().unwrap().is_empty());
    assert!(controller.state().prompt.is_none());
    assert!(controller.state().error.contains("changed"));
}

#[tokio::test]
async fn an_inhibitor_arriving_after_review_prevents_the_session_action() {
    let (controller, backend) = fixture().await;
    let prompt = prepare(&controller).await;
    backend.hardware.lock().unwrap().session.suspend = "inhibited".into();
    controller
        .handle(Request::Confirm {
            token: prompt.token,
        })
        .await
        .unwrap();
    settle(&controller).await;
    assert!(backend.calls.lock().unwrap().is_empty());
    assert!(controller.state().error.contains("inhibited"));
}

#[tokio::test]
async fn closing_while_reading_cannot_create_an_invisible_prompt_or_late_mutation() {
    let (controller, backend) = fixture().await;
    let gate = Arc::new(Notify::new());
    *backend.snapshot_gate.lock().unwrap() = Some(gate.clone());
    controller
        .handle(Request::Prepare {
            kind: SessionAction::Suspend,
        })
        .await
        .unwrap();
    while backend.snapshots.load(Ordering::SeqCst) < 2 {
        tokio::task::yield_now().await;
    }
    controller.set_active(false);
    controller.set_active(true);
    gate.notify_one();
    settle(&controller).await;
    assert!(controller.state().prompt.is_none());
    assert!(backend.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn overlapping_mutations_are_rejected_and_failures_release_admission() {
    let (controller, backend) = fixture().await;
    let gate = Arc::new(Notify::new());
    *backend.apply_gate.lock().unwrap() = Some(gate.clone());
    *backend.fail.lock().unwrap() = Some("Rejected\n".repeat(100));
    controller
        .handle(Request::Profile {
            profile: Profile::PowerSaver,
        })
        .await
        .unwrap();
    while backend.calls.lock().unwrap().is_empty() {
        tokio::task::yield_now().await;
    }
    assert!(controller
        .handle(Request::Profile {
            profile: Profile::Balanced
        })
        .await
        .is_err());
    gate.notify_one();
    settle(&controller).await;
    assert_eq!(backend.calls.lock().unwrap().len(), 1);
    assert!(!controller.state().busy);
    assert_eq!(controller.state().error.len(), 256);
    assert!(!controller.state().error.contains('\n'));
}

#[tokio::test]
async fn unavailable_or_closed_services_never_reach_a_backend() {
    let (controller, backend) = fixture().await;
    controller.set_active(false);
    assert!(controller
        .handle(Request::Profile {
            profile: Profile::Balanced
        })
        .await
        .is_err());
    controller.offline("Private bus disconnected");
    assert!(controller
        .handle(Request::Prepare {
            kind: SessionAction::Suspend
        })
        .await
        .is_err());
    assert!(!controller.state().hardware.session.available);
    assert!(backend.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_disconnected_backend_cannot_publish_a_late_snapshot() {
    let (controller, backend) = fixture().await;
    let gate = Arc::new(Notify::new());
    *backend.snapshot_gate.lock().unwrap() = Some(gate.clone());
    let task = {
        let controller = controller.clone();
        let backend = backend.clone();
        tokio::spawn(async move { controller.refresh(backend.as_ref()).await })
    };
    while backend.snapshots.load(Ordering::SeqCst) < 2 {
        tokio::task::yield_now().await;
    }
    controller.offline("Bus disconnected");
    gate.notify_one();
    assert!(task.await.unwrap().is_err());
    assert!(!controller.state().hardware.session.available);
    assert_eq!(controller.state().error, "Bus disconnected");
}

#[tokio::test]
async fn the_last_disconnected_view_revokes_review() {
    let (controller, _) = fixture().await;
    let first = controller.observe_ui();
    let second = controller.observe_ui();
    let prompt = prepare(&controller).await;
    drop(first);
    assert!(controller.state().prompt.is_some());
    drop(second);
    assert!(controller.state().prompt.is_none());
    assert!(controller
        .handle(Request::Confirm {
            token: prompt.token
        })
        .await
        .is_err());
}
