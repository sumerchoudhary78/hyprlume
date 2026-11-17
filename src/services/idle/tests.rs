use super::*;
use crate::config::Idle;

fn fixture() -> (Arc<Controller>, Credential) {
    let controller = Controller::new("/unused/test.sock".into(), true);
    let token = Credential("a".repeat(64));
    let mut runtime = controller.runtime.lock().unwrap();
    runtime.token = Some(token.clone());
    runtime.timers = true;
    drop(runtime);
    (controller, token)
}

#[test]
fn repeated_resume_is_idempotent_and_old_queued_stages_are_invalidated() {
    let (controller, token) = fixture();
    let mut queue = controller.receiver();
    controller.event(token.clone(), Event::Dim).unwrap();
    controller.event(token.clone(), Event::Dim).unwrap();
    let dim = queue.try_recv().unwrap();
    assert!(queue.try_recv().is_err());
    controller.event(token.clone(), Event::Resume).unwrap();
    assert!(!controller.current(dim.epoch));
    let epoch = *controller.epoch.borrow();
    controller.event(token.clone(), Event::Resume).unwrap();
    assert_eq!(*controller.epoch.borrow(), epoch);
    controller.event(token, Event::Lock).unwrap();
    assert!(controller.current(queue.try_recv().unwrap().epoch));
}

#[test]
fn presentation_revokes_the_old_process_and_sleep_hooks_bypass_the_queue() {
    let (controller, token) = fixture();
    assert!(controller.event(token.clone(), Event::BeforeSleep).unwrap());
    assert!(controller.event(token.clone(), Event::ForceLock).unwrap());
    controller.handle(Request::Presentation { enabled: true });
    assert!(controller.event(token.clone(), Event::Dim).is_err());
    assert!(controller.event(token.clone(), Event::AfterResume).is_err());
    let replacement = Credential("b".repeat(64));
    controller.runtime.lock().unwrap().token = Some(replacement.clone());
    assert!(controller.event(replacement.clone(), Event::Off).is_err());
    assert!(controller.event(replacement, Event::BeforeSleep).unwrap());
    assert!(controller.state().presentation);
    assert!(!serde_json::to_string(&controller.state())
        .unwrap()
        .contains(&token.0));
}

#[test]
fn policy_defaults_are_opt_in_and_invalid_schedules_are_rejected() {
    let mut policy = Idle::default();
    assert!(!policy.enabled);
    assert_eq!(policy.suspend_after_seconds, 0);
    assert!(policy.validate().is_ok());
    policy.screen_off_after_seconds = 200;
    assert!(policy.validate().is_err());
    policy.lock_after_seconds = 0;
    assert!(policy.validate().is_ok());
    policy.dim_after_seconds = 1;
    assert!(policy.validate().is_err());
    policy.dim_after_seconds = 0;
    policy.suspend_after_seconds = u32::MAX;
    assert!(policy.validate().is_err());
}

#[tokio::test]
async fn activity_cancels_a_waiter_without_a_polling_timer() {
    let (controller, token) = fixture();
    let epoch = *controller.epoch.borrow();
    controller.event(token.clone(), Event::Lock).unwrap();
    let waiter = controller.clone();
    let task = tokio::spawn(async move { waiter.cancelled(epoch).await });
    controller.event(token, Event::Resume).unwrap();
    tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
}
