use super::*;
use crate::services::test_bus::Bus;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::Notify;
use zbus::{object_server::SignalEmitter, zvariant::Value};

const COLLECTION: &str = "/org/freedesktop/secrets/collection/test";
const PROMPT: &str = "/org/freedesktop/secrets/prompt/test";

#[derive(Default)]
struct Shared {
    locked: AtomicBool,
    wait_for_auth: AtomicBool,
    prompted: Notify,
    dismissed: AtomicBool,
}

struct SecretService(Arc<Shared>);

#[zbus::interface(name = "org.freedesktop.Secret.Service")]
impl SecretService {
    fn read_alias(&self, name: &str) -> OwnedObjectPath {
        if matches!(name, "login" | "default") {
            COLLECTION
        } else {
            "/"
        }
        .try_into()
        .unwrap()
    }
    fn lock(&self, objects: Vec<OwnedObjectPath>) -> (Vec<OwnedObjectPath>, OwnedObjectPath) {
        assert_eq!(objects[0].as_str(), COLLECTION);
        self.0.locked.store(true, Ordering::SeqCst);
        (objects, "/".try_into().unwrap())
    }
    fn unlock(&self, objects: Vec<OwnedObjectPath>) -> (Vec<OwnedObjectPath>, OwnedObjectPath) {
        assert_eq!(objects[0].as_str(), COLLECTION);
        (vec![], PROMPT.try_into().unwrap())
    }
}

struct SecretCollection(Arc<Shared>);

#[zbus::interface(name = "org.freedesktop.Secret.Collection")]
impl SecretCollection {
    #[zbus(property)]
    fn locked(&self) -> bool {
        self.0.locked.load(Ordering::SeqCst)
    }
}

struct SecretPrompt(Arc<Shared>);

#[zbus::interface(name = "org.freedesktop.Secret.Prompt")]
impl SecretPrompt {
    async fn prompt(&self, window_id: &str, #[zbus(signal_emitter)] signal: SignalEmitter<'_>) {
        assert!(window_id.is_empty());
        self.0.prompted.notify_one();
        if !self.0.wait_for_auth.load(Ordering::SeqCst) {
            self.0.locked.store(false, Ordering::SeqCst);
            Self::completed(
                &signal,
                false,
                Value::new(vec![OwnedObjectPath::try_from(COLLECTION).unwrap()]),
            )
            .await
            .unwrap();
        }
    }
    fn dismiss(&self) {
        self.0.dismissed.store(true, Ordering::SeqCst);
    }
    #[zbus(signal)]
    async fn completed(
        signal: &SignalEmitter<'_>,
        dismissed: bool,
        result: Value<'_>,
    ) -> zbus::Result<()>;
}

async fn provider(bus: &Bus, data: &Arc<Shared>) -> Connection {
    zbus::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name(SERVICE)
        .unwrap()
        .serve_at(PATH, SecretService(data.clone()))
        .unwrap()
        .serve_at(COLLECTION, SecretCollection(data.clone()))
        .unwrap()
        .serve_at(PROMPT, SecretPrompt(data.clone()))
        .unwrap()
        .build()
        .await
        .unwrap()
}

#[tokio::test]
async fn reads_only_metadata_and_verifies_provider_prompt_completion() {
    let bus = Bus::new().await;
    let connection = bus.connect().await;
    let locking = Controller::new(true);
    let read = handle_on(&connection, &locking, 0, Request::Refresh)
        .await
        .unwrap();
    assert!(matches!(read.provider, Provider::Missing));
    let shared = Arc::new(Shared::default());
    let _provider = provider(&bus, &shared).await;
    let locked = handle_on(
        &connection,
        &locking,
        locking.epoch(),
        Request::Lock {
            collection: Collection::Default,
        },
    )
    .await
    .unwrap();
    assert_eq!(locked.default_locked, Some(true));
    let unlocked = handle_on(
        &connection,
        &locking,
        locking.epoch(),
        Request::Unlock {
            collection: Collection::Login,
        },
    )
    .await
    .unwrap();
    assert_eq!(unlocked.login_locked, Some(false));
    assert!(unlocked.error.is_empty());
}

#[tokio::test]
async fn dismisses_pending_provider_prompt_if_the_desktop_locks() {
    let bus = Bus::new().await;
    let connection = bus.connect().await;
    let shared = Arc::new(Shared::default());
    shared.locked.store(true, Ordering::SeqCst);
    shared.wait_for_auth.store(true, Ordering::SeqCst);
    let _provider = provider(&bus, &shared).await;
    let locking = Controller::new(true);
    let epoch = locking.epoch();
    let lock_while_waiting = async {
        shared.prompted.notified().await;
        locking.simulate(true);
    };
    let operation = handle_on(
        &connection,
        &locking,
        epoch,
        Request::Unlock {
            collection: Collection::Login,
        },
    );
    let (_, result) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(lock_while_waiting, operation)
    })
    .await
    .unwrap();
    assert!(result.unwrap().error.contains("dismissed"));
    assert!(shared.dismissed.load(Ordering::SeqCst));
    assert!(shared.locked.load(Ordering::SeqCst));
}
