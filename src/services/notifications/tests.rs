use super::{
    store::{Store, MAX_ITEMS},
    types::{Incoming, Signal},
    Request,
};
use std::time::{Duration, Instant};

fn incoming(timeout_ms: i32) -> Incoming {
    Incoming {
        app_name: "Test app".into(),
        summary: "Build finished".into(),
        body: "<b>Plain text only</b>".into(),
        actions: vec!["default".into(), "Open".into()],
        urgency: 1,
        timeout_ms,
        ..Default::default()
    }
}

#[test]
fn replacements_keep_ids_refresh_deadlines_and_invalidate_old_actions() {
    let mut store = Store::default();
    let now = Instant::now();
    let (id, _) = store.receive(":1.1", incoming(10), now).unwrap();
    let old = store.snapshot().items[0].revision;
    let replacement = Incoming {
        replaces_id: id,
        ..incoming(500)
    };
    assert_eq!(store.receive(":1.1", replacement, now).unwrap().0, id);
    assert_eq!(store.snapshot().items.len(), 1);
    assert!(store
        .request(Request::Invoke {
            id,
            revision: old,
            key: "default".into()
        })
        .is_err());
    assert!(store.expire(now + Duration::from_millis(11)).is_empty());
    assert_eq!(
        store.expire(now + Duration::from_millis(501)),
        vec![Signal::Closed {
            owner: ":1.1".into(),
            id,
            reason: 1
        }]
    );
    assert!(store.expire(now + Duration::from_secs(1)).is_empty());
}

#[test]
fn senders_cannot_replace_or_close_other_applications_notifications() {
    let mut store = Store::default();
    let (first, _) = store.receive(":1.1", incoming(0), Instant::now()).unwrap();
    let replacement = Incoming {
        replaces_id: first,
        ..incoming(0)
    };
    let (second, _) = store.receive(":1.2", replacement, Instant::now()).unwrap();
    assert_ne!(first, second);
    assert!(store.close_from_client(":1.2", first).is_err());
    assert_eq!(
        store.close_from_client(":1.1", first).unwrap(),
        vec![Signal::Closed {
            owner: ":1.1".into(),
            id: first,
            reason: 3
        }]
    );
    assert!(store.close_from_client(":1.1", first).is_err());
}

#[test]
fn actions_use_current_keys_once_and_resident_notifications_stay_active() {
    let mut store = Store::default();
    let (id, _) = store.receive(":1.1", incoming(0), Instant::now()).unwrap();
    let revision = store.snapshot().items[0].revision;
    assert!(store
        .request(Request::Invoke {
            id,
            revision,
            key: "exec:sh".into()
        })
        .is_err());
    assert_eq!(
        store
            .request(Request::Invoke {
                id,
                revision,
                key: "default".into()
            })
            .unwrap(),
        vec![
            Signal::Action {
                owner: ":1.1".into(),
                id,
                key: "default".into()
            },
            Signal::Closed {
                owner: ":1.1".into(),
                id,
                reason: 2
            }
        ]
    );
    assert!(store
        .request(Request::Invoke {
            id,
            revision,
            key: "default".into()
        })
        .is_err());
    let (id, _) = store
        .receive(
            ":1.1",
            Incoming {
                resident: true,
                ..incoming(-1)
            },
            Instant::now(),
        )
        .unwrap();
    let revision = store.snapshot().items[0].revision;
    assert_eq!(
        store
            .request(Request::Invoke {
                id,
                revision,
                key: "default".into()
            })
            .unwrap()
            .len(),
        1
    );
    assert!(store.snapshot().items[0].active);
    assert!(store.deadline().is_none());
}

#[test]
fn dnd_suppresses_popups_without_replaying_them_and_opening_marks_read() {
    let mut store = Store::default();
    store.request(Request::Dnd { enabled: true }).unwrap();
    store.receive(":1.1", incoming(0), Instant::now()).unwrap();
    assert!(!store.snapshot().items[0].popup);
    assert_eq!(store.snapshot().unread, 1);
    store.request(Request::Dnd { enabled: false }).unwrap();
    assert!(!store.snapshot().items[0].popup);
    store.request(Request::Toggle).unwrap();
    assert_eq!(store.snapshot().unread, 0);
    store.receive(":1.1", incoming(0), Instant::now()).unwrap();
    assert!(store
        .snapshot()
        .items
        .iter()
        .all(|item| item.read && !item.popup));
}

#[test]
fn critical_notifications_do_not_expire_and_transient_ones_leave_no_history() {
    let mut store = Store::default();
    let now = Instant::now();
    store
        .receive(
            ":1.1",
            Incoming {
                urgency: 2,
                ..incoming(-1)
            },
            now,
        )
        .unwrap();
    store
        .receive(
            ":1.1",
            Incoming {
                transient: true,
                ..incoming(10)
            },
            now,
        )
        .unwrap();
    store.expire(now + Duration::from_secs(60));
    assert_eq!(store.snapshot().items.len(), 1);
    assert!(store.snapshot().items[0].active);
    assert!(store.deadline().is_none());
}

#[test]
fn dismiss_remove_and_clear_emit_exactly_one_close_per_active_notice() {
    let mut store = Store::default();
    let (id, _) = store.receive(":1.1", incoming(0), Instant::now()).unwrap();
    let revision = store.snapshot().items[0].revision;
    assert_eq!(
        store
            .request(Request::Dismiss { id, revision })
            .unwrap()
            .len(),
        1
    );
    assert!(!store.snapshot().items[0].active);
    assert!(store
        .request(Request::Remove { id, revision })
        .unwrap()
        .is_empty());
    assert!(store.snapshot().items.is_empty());
    store.receive(":1.1", incoming(0), Instant::now()).unwrap();
    store.receive(":1.2", incoming(0), Instant::now()).unwrap();
    assert_eq!(store.request(Request::Clear).unwrap().len(), 2);
    assert!(store.snapshot().items.is_empty());
}

#[test]
fn history_popups_and_wire_size_are_bounded_even_with_adversarial_text() {
    let mut store = Store::default();
    let mut closed = 0;
    for _ in 0..40 {
        let input = Incoming {
            app_name: "\\".repeat(1024),
            summary: "\"".repeat(8192),
            body: "\n".repeat(32768),
            actions: (0..6)
                .map(|index| {
                    if index % 2 == 0 {
                        "k".repeat(64)
                    } else {
                        "\\".repeat(1000)
                    }
                })
                .collect(),
            ..incoming(0)
        };
        closed += store
            .receive(":1.1", input, Instant::now())
            .unwrap()
            .1
            .len();
    }
    let state = store.snapshot();
    assert_eq!(state.items.len(), MAX_ITEMS);
    assert_eq!(state.items.iter().filter(|item| item.popup).count(), 3);
    assert_eq!(closed, 40 - MAX_ITEMS);
    assert!(serde_json::to_vec(&state).unwrap().len() < 60 * 1024);
}

#[test]
fn malformed_inputs_do_not_mutate_history_or_truncate_action_identifiers() {
    let mut store = Store::default();
    let malformed = Incoming {
        actions: vec!["unpaired key".into()],
        ..incoming(0)
    };
    assert!(store.receive(":1.1", malformed, Instant::now()).is_err());
    assert!(store
        .receive(
            ":1.1",
            Incoming {
                body: "x".repeat(32769),
                ..incoming(0)
            },
            Instant::now()
        )
        .is_err());
    assert!(store.snapshot().items.is_empty());
    store
        .receive(
            ":1.1",
            Incoming {
                actions: vec!["a".repeat(65), "label".into()],
                ..incoming(0)
            },
            Instant::now(),
        )
        .unwrap();
    assert!(store.snapshot().items[0].actions.is_empty());
}

#[test]
fn a_lost_bus_keeps_history_but_invalidates_actions_popups_and_timers() {
    let mut store = Store::default();
    let (id, _) = store
        .receive(":1.1", incoming(2000), Instant::now())
        .unwrap();
    let revision = store.snapshot().items[0].revision;
    store.offline("Session bus disconnected");
    let state = store.snapshot();
    assert_eq!(state.items.len(), 1);
    assert!(!state.available && !state.items[0].popup && !state.items[0].active);
    assert!(store.deadline().is_none());
    assert!(store
        .request(Request::Invoke {
            id,
            revision,
            key: "default".into()
        })
        .is_err());
}
