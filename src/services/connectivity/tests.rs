use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex as StdMutex,
};
use tokio::sync::Notify;

struct Fake {
    hardware: Hardware,
    calls: StdMutex<Vec<Request>>,
    stops: AtomicUsize,
    gate: Option<Arc<Notify>>,
    fail: Option<String>,
}

impl Backend for Fake {
    fn snapshot(&self) -> BoxFuture<'_, Result<Hardware>> {
        Box::pin(async { Ok(self.hardware.clone()) })
    }
    fn execute(&self, request: Request, _state: State) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(request);
            if let Some(gate) = &self.gate {
                gate.notified().await;
            }
            if let Some(message) = &self.fail {
                anyhow::bail!("{message}");
            }
            Ok(())
        })
    }
    fn stop_discovery(&self) -> BoxFuture<'_, ()> {
        Box::pin(async {
            self.stops.fetch_add(1, Ordering::Relaxed);
        })
    }
}

async fn fixture(gate: Option<Arc<Notify>>, fail: Option<String>) -> (Arc<Controller>, Arc<Fake>) {
    let controller = Controller::new(true);
    let backend = Arc::new(Fake {
        hardware: Hardware {
            wifi: WifiState {
                available: true,
                enabled: true,
                hardware_enabled: true,
                interfaces: vec![WifiInterface {
                    name: "wlan0".into(),
                    ..Default::default()
                }],
                networks: vec![WifiNetwork {
                    id: "lab".into(),
                    interface: "wlan0".into(),
                    ssid: "Lab".into(),
                    security: "wpa2".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            bluetooth: BluetoothState {
                available: true,
                adapters: vec![BluetoothAdapter {
                    id: "hci0".into(),
                    powered: true,
                    ..Default::default()
                }],
                devices: vec![BluetoothDevice {
                    adapter: "hci0".into(),
                    address: "AA:BB:CC:DD:EE:FF".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        },
        calls: StdMutex::new(Vec::new()),
        stops: AtomicUsize::new(0),
        gate,
        fail,
    });
    assert!(controller.backend.set(backend.clone()).is_ok());
    controller.refresh().await;
    (controller, backend)
}

async fn idle(controller: &Controller) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while controller.state().busy {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

async fn prompt(controller: &Controller) -> PairPrompt {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(prompt) = controller.state().prompt {
                break prompt;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn stale_targets_never_reach_a_backend_and_credentials_never_reach_state() {
    let secret = "fixture_wifi_secret";
    let (controller, backend) = fixture(None, Some(format!("Backend rejected {secret}"))).await;
    assert!(controller
        .handle(Request::WifiConnect {
            network: "stale".into(),
            password: Some(Secret::new(secret.into()))
        })
        .await
        .is_err());
    assert!(backend.calls.lock().unwrap().is_empty());
    let request = Request::WifiConnect {
        network: "lab".into(),
        password: Some(Secret::new(secret.into())),
    };
    assert!(!format!("{request:?}").contains(secret));
    let wire = serde_json::to_string(&request).unwrap();
    assert!(
        wire.contains(secret),
        "the private wire must carry the actual credential"
    );
    controller
        .handle(serde_json::from_str(&wire).unwrap())
        .await
        .unwrap();
    idle(&controller).await;
    let state = controller.state();
    assert!(state.error.contains("[credential]"));
    assert!(!serde_json::to_string(&state).unwrap().contains(secret));
    assert_eq!(backend.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn an_in_flight_operation_is_not_queued_again() {
    let gate = Arc::new(Notify::new());
    let (controller, backend) = fixture(Some(gate.clone()), None).await;
    controller
        .handle(Request::WifiScan {
            interface: Some("wlan0".into()),
        })
        .await
        .unwrap();
    assert!(controller.state().busy);
    assert!(controller
        .handle(Request::WifiPower { enabled: false })
        .await
        .is_err());
    controller.handle(Request::Close).await.unwrap();
    assert_eq!(backend.stops.load(Ordering::Relaxed), 1);
    gate.notify_one();
    idle(&controller).await;
    assert_eq!(backend.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn closing_the_panel_stops_discovery_without_switching_radios() {
    let (controller, backend) = fixture(None, None).await;
    controller.handle(Request::Toggle).await.unwrap();
    assert!(controller.state().panel_open);
    controller.handle(Request::Close).await.unwrap();
    assert!(!controller.state().panel_open);
    assert_eq!(backend.stops.load(Ordering::Relaxed), 1);
    assert!(backend.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn closing_before_a_pairing_prompt_cancels_the_pair_operation() {
    let (controller, backend) = fixture(Some(Arc::new(Notify::new())), None).await;
    controller.handle(Request::Toggle).await.unwrap();
    controller
        .handle(Request::BluetoothPair {
            adapter: "hci0".into(),
            address: "AA:BB:CC:DD:EE:FF".into(),
            trust: false,
        })
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while backend.calls.lock().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    controller.handle(Request::Toggle).await.unwrap();
    idle(&controller).await;
    assert!(!controller.state().panel_open);
    assert!(controller.state().prompt.is_none());
    assert_eq!(backend.stops.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn pairing_replies_are_single_use_and_bound_to_the_current_token() {
    let controller = Controller::new(true);
    let pairing = controller.pairing.clone();
    let task = tokio::spawn(async move {
        pairing
            .ask(
                "hci0".into(),
                "AA:BB:CC:DD:EE:FF".into(),
                "confirm",
                "042619".into(),
            )
            .await
    });
    let active = prompt(&controller).await;
    assert!(
        !controller.state().panel_open,
        "a late BlueZ callback must not reopen a dismissed panel"
    );
    assert!(controller
        .pairing
        .reply(active.token + 1, true, None)
        .is_err());
    assert!(controller.state().prompt.is_some());
    controller.pairing.reply(active.token, true, None).unwrap();
    assert!(task.await.unwrap().unwrap().is_none());
    assert!(controller.state().prompt.is_none());
    assert!(controller.pairing.reply(active.token, true, None).is_err());
}

#[tokio::test]
async fn cancelled_or_expired_pairing_prompts_are_removed() {
    let controller = Controller::new(true);
    let pairing = controller.pairing.clone();
    let task = tokio::spawn(async move {
        pairing
            .ask(
                "hci0".into(),
                "AA:BB:CC:DD:EE:FF".into(),
                "pin",
                String::new(),
            )
            .await
    });
    let active = prompt(&controller).await;
    task.abort();
    let _ = task.await;
    assert!(controller.state().prompt.is_none());
    assert!(controller.pairing.reply(active.token, true, None).is_err());
    let short = pairing::Pairing::new(controller.state.clone(), Duration::from_millis(20));
    assert!(short
        .ask(
            "hci0".into(),
            "AA:BB:CC:DD:EE:FF".into(),
            "pin",
            String::new()
        )
        .await
        .is_err());
    assert!(controller.state().prompt.is_none());
}

#[test]
fn bounded_snapshots_prioritize_active_devices_and_fit_the_reply_frame() {
    let mut hardware = Hardware::default();
    hardware.wifi.networks = (0..200)
        .map(|index| WifiNetwork {
            id: format!("network{index}"),
            ssid: "\u{0001}".repeat(32),
            strength: (index % 101) as u8,
            connected: index == 199,
            ..Default::default()
        })
        .collect();
    hardware.bluetooth.devices = (0..200)
        .map(|index| BluetoothDevice {
            address: format!("{index:017}"),
            name: "\u{0001}".repeat(4096),
            connected: index == 199,
            ..Default::default()
        })
        .collect();
    let bounded = hardware.bounded();
    assert_eq!(bounded.wifi.networks.len(), 32);
    assert!(bounded.wifi.networks[0].connected);
    assert_eq!(bounded.bluetooth.devices.len(), 32);
    assert!(bounded.bluetooth.devices[0].connected);
    let state = State {
        hardware: bounded,
        ..Default::default()
    };
    assert!(serde_json::to_vec(&state).unwrap().len() < 64 * 1024);
}
