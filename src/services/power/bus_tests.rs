//! Native adapters are exercised on a private bus with synthetic hardware.
use super::{native, types::Mutation, Backend, Controller, Profile, Request, SessionAction};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use zbus::{
    fdo,
    zvariant::{OwnedObjectPath, OwnedValue, Str},
    Connection,
};

const SESSION: &str = "/org/freedesktop/login1/session/_7";
const BATTERY: &str = "/org/freedesktop/UPower/devices/DisplayDevice";
const PROFILES: &str = "/org/freedesktop/UPower/PowerProfiles";

struct Bus {
    address: String,
    _child: Child,
    _directory: tempfile::TempDir,
}
impl Bus {
    async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("bus.conf");
        std::fs::write(&config, format!("<busconfig><type>session</type><listen>unix:tmpdir={}</listen><auth>EXTERNAL</auth><policy context=\"default\"><allow send_destination=\"*\"/><allow receive_sender=\"*\"/><allow own=\"*\"/></policy></busconfig>", directory.path().display())).unwrap();
        let mut child = Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("install dbus-daemon for isolated power tests");
        let mut address = String::new();
        tokio::time::timeout(
            Duration::from_secs(3),
            BufReader::new(child.stdout.take().unwrap()).read_line(&mut address),
        )
        .await
        .unwrap()
        .unwrap();
        Self {
            address: address.trim().into(),
            _child: child,
            _directory: directory,
        }
    }
    async fn connect(&self) -> Connection {
        zbus::connection::Builder::address(self.address.as_str())
            .unwrap()
            .build()
            .await
            .unwrap()
    }
}

struct Shared {
    calls: Mutex<Vec<String>>,
    percentage: Mutex<f64>,
    profile: Mutex<String>,
    remote: AtomicBool,
    active: AtomicBool,
    timestamp: AtomicU64,
    uid: AtomicU64,
    inhibited: AtomicBool,
    second_session: AtomicBool,
    reads: AtomicU64,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            percentage: Mutex::new(72.0),
            profile: Mutex::new("balanced".into()),
            remote: AtomicBool::new(false),
            active: AtomicBool::new(true),
            timestamp: AtomicU64::new(1234),
            uid: AtomicU64::new(1000),
            inhibited: AtomicBool::new(false),
            second_session: AtomicBool::new(false),
            reads: AtomicU64::new(0),
        }
    }
}

struct Manager(Arc<Shared>);
#[zbus::interface(name = "org.freedesktop.login1.Manager")]
impl Manager {
    fn get_session(&self, _id: &str) -> OwnedObjectPath {
        SESSION.try_into().unwrap()
    }
    fn list_sessions(&self) -> Vec<(String, u32, String, String, OwnedObjectPath)> {
        let mut sessions = vec![(
            "7".into(),
            1000,
            "fixture".into(),
            "seat0".into(),
            SESSION.try_into().unwrap(),
        )];
        if self.0.second_session.load(Ordering::SeqCst) {
            sessions.push((
                "8".into(),
                1000,
                "fixture".into(),
                "seat1".into(),
                SESSION.try_into().unwrap(),
            ));
        }
        sessions
    }
    fn can_suspend(&self) -> &str {
        "yes"
    }
    fn can_reboot(&self) -> &str {
        "challenge"
    }
    fn can_power_off(&self) -> &str {
        "yes"
    }
    fn list_inhibitors(&self) -> Vec<(String, String, String, String, u32, u32)> {
        if self.0.inhibited.load(Ordering::SeqCst) {
            vec![(
                "sleep:shutdown".into(),
                "fixture".into(),
                "do not interrupt".into(),
                "block-weak".into(),
                1000,
                42,
            )]
        } else {
            vec![]
        }
    }
    fn suspend_with_flags(&self, flags: u64) {
        self.0
            .calls
            .lock()
            .unwrap()
            .push(format!("suspend:{flags}"));
    }
    fn reboot_with_flags(&self, flags: u64) {
        self.0.calls.lock().unwrap().push(format!("reboot:{flags}"));
    }
    fn power_off_with_flags(&self, flags: u64) {
        self.0
            .calls
            .lock()
            .unwrap()
            .push(format!("power_off:{flags}"));
    }
}

struct Login {
    shared: Arc<Shared>,
    root: PathBuf,
}
#[zbus::interface(name = "org.freedesktop.login1.Session")]
impl Login {
    fn terminate(&self) {
        self.shared.calls.lock().unwrap().push("logout:7".into());
    }
    fn set_brightness(&self, subsystem: &str, name: &str, value: u32) -> fdo::Result<()> {
        if subsystem != "backlight" || name != "sample_panel" || value > 1000 {
            return Err(fdo::Error::InvalidArgs("invalid fixture backlight".into()));
        }
        std::fs::write(self.root.join("sample_panel/brightness"), value.to_string())
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        self.shared
            .calls
            .lock()
            .unwrap()
            .push(format!("brightness:{name}:{value}"));
        Ok(())
    }
    #[zbus(property)]
    fn id(&self) -> &str {
        "7"
    }
    #[zbus(property)]
    fn user(&self) -> (u32, OwnedObjectPath) {
        (
            self.shared.uid.load(Ordering::SeqCst) as u32,
            "/org/freedesktop/login1/user/_1000".try_into().unwrap(),
        )
    }
    #[zbus(property)]
    fn remote(&self) -> bool {
        self.shared.remote.load(Ordering::SeqCst)
    }
    #[zbus(property)]
    fn active(&self) -> bool {
        self.shared.active.load(Ordering::SeqCst)
    }
    #[zbus(property, name = "Type")]
    fn session_type(&self) -> &str {
        "wayland"
    }
    #[zbus(property)]
    fn class(&self) -> &str {
        "user"
    }
    #[zbus(property)]
    fn state(&self) -> &str {
        "active"
    }
    #[zbus(property)]
    fn timestamp_monotonic(&self) -> u64 {
        self.shared.timestamp.load(Ordering::SeqCst)
    }
}

struct Battery(Arc<Shared>);
#[zbus::interface(name = "org.freedesktop.UPower.Device")]
impl Battery {
    #[zbus(property)]
    fn is_present(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn percentage(&self) -> f64 {
        self.0.reads.fetch_add(1, Ordering::SeqCst);
        *self.0.percentage.lock().unwrap()
    }
    #[zbus(property)]
    fn state(&self) -> u32 {
        2
    }
    #[zbus(property)]
    fn time_to_empty(&self) -> i64 {
        7200
    }
    #[zbus(property)]
    fn time_to_full(&self) -> i64 {
        0
    }
}

struct Profiles(Arc<Shared>);
#[zbus::interface(name = "org.freedesktop.UPower.PowerProfiles")]
impl Profiles {
    #[zbus(property)]
    fn active_profile(&self) -> String {
        self.0.profile.lock().unwrap().clone()
    }
    #[zbus(property)]
    fn set_active_profile(&self, profile: String) {
        *self.0.profile.lock().unwrap() = profile.clone();
        self.0
            .calls
            .lock()
            .unwrap()
            .push(format!("profile:{profile}"));
    }
    #[zbus(property)]
    fn performance_degraded(&self) -> &str {
        ""
    }
    #[zbus(property)]
    fn profiles(&self) -> Vec<HashMap<String, OwnedValue>> {
        ["power-saver", "balanced", "performance"]
            .into_iter()
            .map(|profile| {
                HashMap::from([("Profile".into(), OwnedValue::from(Str::from(profile)))])
            })
            .collect()
    }
}

struct Fixture {
    bus: Bus,
    server: Connection,
    shared: Arc<Shared>,
    root: tempfile::TempDir,
    native: Arc<native::Native>,
}
impl Fixture {
    async fn new() -> Self {
        let bus = Bus::new().await;
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("sample_panel")).unwrap();
        std::fs::write(root.path().join("sample_panel/max_brightness"), "1000").unwrap();
        std::fs::write(root.path().join("sample_panel/brightness"), "600").unwrap();
        let shared = Arc::new(Shared::default());
        let server = zbus::connection::Builder::address(bus.address.as_str())
            .unwrap()
            .serve_at("/org/freedesktop/login1", Manager(shared.clone()))
            .unwrap()
            .serve_at(
                SESSION,
                Login {
                    shared: shared.clone(),
                    root: root.path().into(),
                },
            )
            .unwrap()
            .serve_at(BATTERY, Battery(shared.clone()))
            .unwrap()
            .serve_at(PROFILES, Profiles(shared.clone()))
            .unwrap()
            .name("org.freedesktop.login1")
            .unwrap()
            .name("org.freedesktop.UPower")
            .unwrap()
            .name("org.freedesktop.UPower.PowerProfiles")
            .unwrap()
            .build()
            .await
            .unwrap();
        let native = Arc::new(native::Native::new(
            bus.connect().await,
            root.path().into(),
            Some("7".into()),
            1000,
        ));
        Self {
            bus,
            server,
            shared,
            root,
            native,
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_properties_profiles_brightness_and_session_calls_use_private_services() {
    let fixture = Fixture::new().await;
    let hardware = fixture.native.snapshot().await.unwrap();
    assert_eq!(hardware.battery.percentage, Some(72.0));
    assert_eq!(hardware.battery.seconds_remaining, Some(7200));
    assert_eq!(hardware.profiles.active, Some(Profile::Balanced));
    assert_eq!(hardware.backlights[0].percent, 60);
    assert!(hardware.session.available);
    let light = hardware.backlights[0].id.clone();
    fixture
        .native
        .apply(Mutation::Profile(Profile::PowerSaver), hardware.clone())
        .await
        .unwrap();
    fixture
        .native
        .apply(
            Mutation::Brightness {
                device: light.clone(),
                percent: 37,
            },
            hardware.clone(),
        )
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.root.path().join("sample_panel/brightness")).unwrap(),
        "370"
    );
    fixture
        .native
        .apply(
            Mutation::BrightnessRestore {
                device: light.clone(),
                expected: 111,
                value: 600,
            },
            hardware.clone(),
        )
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.root.path().join("sample_panel/brightness")).unwrap(),
        "370"
    );
    fixture
        .native
        .apply(
            Mutation::BrightnessRestore {
                device: light,
                expected: 370,
                value: 600,
            },
            hardware.clone(),
        )
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.root.path().join("sample_panel/brightness")).unwrap(),
        "600"
    );
    assert_eq!(
        fixture.native.snapshot().await.unwrap().profiles.active,
        Some(Profile::PowerSaver)
    );
    for kind in [
        SessionAction::Suspend,
        SessionAction::Reboot,
        SessionAction::PowerOff,
        SessionAction::Logout,
    ] {
        fixture
            .native
            .apply(
                Mutation::Session {
                    kind,
                    target: hardware.session.target.clone().unwrap(),
                },
                hardware.clone(),
            )
            .await
            .unwrap();
    }
    assert_eq!(
        *fixture.shared.calls.lock().unwrap(),
        [
            "profile:power-saver",
            "brightness:sample_panel:370",
            "brightness:sample_panel:600",
            "suspend:1",
            "reboot:1",
            "power_off:1",
            "logout:7"
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_other_user_ambiguous_and_replaced_sessions_cannot_be_controlled() {
    let fixture = Fixture::new().await;
    let observed = fixture.native.snapshot().await.unwrap();
    fixture.shared.remote.store(true, Ordering::SeqCst);
    assert!(!fixture.native.snapshot().await.unwrap().session.available);
    assert!(fixture
        .native
        .apply(
            Mutation::Session {
                kind: SessionAction::Logout,
                target: observed.session.target.clone().unwrap()
            },
            observed.clone()
        )
        .await
        .is_err());
    fixture.shared.remote.store(false, Ordering::SeqCst);
    fixture.shared.uid.store(1001, Ordering::SeqCst);
    assert!(!fixture.native.snapshot().await.unwrap().session.available);
    fixture.shared.uid.store(1000, Ordering::SeqCst);
    fixture.shared.timestamp.store(9999, Ordering::SeqCst);
    assert!(fixture
        .native
        .apply(
            Mutation::Session {
                kind: SessionAction::Logout,
                target: observed.session.target.clone().unwrap()
            },
            observed.clone()
        )
        .await
        .is_err());
    fixture.shared.timestamp.store(1234, Ordering::SeqCst);
    fixture.shared.second_session.store(true, Ordering::SeqCst);
    let ambiguous = native::Native::new(
        fixture.bus.connect().await,
        fixture.root.path().into(),
        None,
        1000,
    );
    assert!(!ambiguous.snapshot().await.unwrap().session.available);
    assert!(fixture.shared.calls.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn inhibitors_and_a_removed_backlight_are_rechecked_before_mutation() {
    let fixture = Fixture::new().await;
    let observed = fixture.native.snapshot().await.unwrap();
    fixture.shared.inhibited.store(true, Ordering::SeqCst);
    assert_eq!(
        fixture.native.snapshot().await.unwrap().session.suspend,
        "inhibited"
    );
    assert!(fixture
        .native
        .apply(
            Mutation::Session {
                kind: SessionAction::Suspend,
                target: observed.session.target.clone().unwrap()
            },
            observed.clone()
        )
        .await
        .is_err());
    std::fs::remove_dir_all(fixture.root.path().join("sample_panel")).unwrap();
    assert!(fixture
        .native
        .apply(
            Mutation::Brightness {
                device: observed.backlights[0].id.clone(),
                percent: 40
            },
            observed.clone()
        )
        .await
        .is_err());
    assert!(fixture.shared.calls.lock().unwrap().is_empty());
}

async fn wait(controller: &Controller, check: impl Fn(&super::State) -> bool) {
    tokio::time::timeout(Duration::from_secs(3), async {
        while !check(&controller.state()) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_events_update_state_and_service_replacement_revokes_old_reviews() {
    let fixture = Fixture::new().await;
    let controller = Controller::new(false);
    let observer = tokio::spawn(native::observe(controller.clone(), fixture.native.clone()));
    wait(&controller, |state| state.hardware.session.available).await;
    controller.set_active(true);
    controller
        .handle(Request::Prepare {
            kind: SessionAction::Suspend,
        })
        .await
        .unwrap();
    wait(&controller, |state| state.prompt.is_some() && !state.busy).await;
    let token = controller.state().prompt.unwrap().token;
    let observed = controller.state().hardware.clone();
    *fixture.shared.percentage.lock().unwrap() = 43.0;
    let interface = fixture
        .server
        .object_server()
        .interface::<_, Battery>(BATTERY)
        .await
        .unwrap();
    interface
        .get()
        .await
        .percentage_changed(interface.signal_emitter())
        .await
        .unwrap();
    wait(&controller, |state| {
        state.hardware.battery.percentage == Some(43.0)
    })
    .await;
    fixture
        .server
        .release_name("org.freedesktop.login1")
        .await
        .unwrap();
    wait(&controller, |state| {
        !state.hardware.session.available && state.prompt.is_none()
    })
    .await;
    assert!(controller.handle(Request::Confirm { token }).await.is_err());
    // The old unique connection still exports its session; it must not be used.
    assert!(fixture
        .native
        .apply(
            Mutation::Session {
                kind: SessionAction::Logout,
                target: observed.session.target.clone().unwrap()
            },
            super::Hardware::default()
        )
        .await
        .is_err());
    let _replacement = zbus::connection::Builder::address(fixture.bus.address.as_str())
        .unwrap()
        .serve_at("/org/freedesktop/login1", Manager(fixture.shared.clone()))
        .unwrap()
        .serve_at(
            SESSION,
            Login {
                shared: fixture.shared.clone(),
                root: fixture.root.path().into(),
            },
        )
        .unwrap()
        .name("org.freedesktop.login1")
        .unwrap()
        .build()
        .await
        .unwrap();
    wait(&controller, |state| state.hardware.session.available).await;
    assert!(fixture
        .native
        .apply(
            Mutation::Session {
                kind: SessionAction::Logout,
                target: observed.session.target.unwrap()
            },
            super::Hardware::default()
        )
        .await
        .is_err());
    assert!(fixture.shared.calls.lock().unwrap().is_empty());
    controller.set_active(false);
    observer.abort();
    let _ = observer.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn closed_panels_do_not_poll_and_visible_brightness_observes_external_changes() {
    let fixture = Fixture::new().await;
    let controller = Controller::new(false);
    let _view = controller.observe_ui();
    let observer = tokio::spawn(native::observe(controller.clone(), fixture.native.clone()));
    wait(&controller, |state| state.hardware.session.available).await;
    let reads = fixture.shared.reads.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(2200)).await;
    assert_eq!(
        fixture.shared.reads.load(Ordering::SeqCst),
        reads,
        "closed service polled D-Bus"
    );
    std::fs::write(fixture.root.path().join("sample_panel/brightness"), "200").unwrap();
    controller.set_active(true);
    wait(&controller, |state| {
        state.hardware.backlights[0].percent == 20
    })
    .await;
    std::fs::write(fixture.root.path().join("sample_panel/brightness"), "350").unwrap();
    wait(&controller, |state| {
        state.hardware.backlights[0].percent == 35
    })
    .await;
    let reads = fixture.shared.reads.load(Ordering::SeqCst);
    controller.set_active(false);
    std::fs::write(fixture.root.path().join("sample_panel/brightness"), "400").unwrap();
    tokio::time::sleep(Duration::from_millis(2200)).await;
    assert_eq!(fixture.shared.reads.load(Ordering::SeqCst), reads);
    assert_eq!(
        controller.state().hardware.backlights[0].percent,
        35,
        "hidden panel continued brightness sampling"
    );
    observer.abort();
    let _ = observer.await;
}
