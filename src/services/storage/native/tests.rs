use super::*;
use crate::services::test_bus::Bus;
use std::sync::{Arc, Mutex};
use zbus::{
    fdo::{Error, ManagedObjects},
    zvariant::{ObjectPath, OwnedValue},
};

const USB: &str = "/org/freedesktop/UDisks2/drives/USB";
const SATA: &str = "/org/freedesktop/UDisks2/drives/SATA";
const USB_BLOCK: &str = "/org/freedesktop/UDisks2/block_devices/sdb1";
const OS_BLOCK: &str = "/org/freedesktop/UDisks2/block_devices/sda1";
const MAPPING: &str = "/org/freedesktop/UDisks2/block_devices/dm_0";
const BLOCK: &str = "org.freedesktop.UDisks2.Block";
const DRIVE: &str = "org.freedesktop.UDisks2.Drive";

#[derive(Default)]
struct Fake {
    calls: Vec<&'static str>,
    unlocked: bool,
    mounted: bool,
    busy: bool,
    removed: bool,
}
type Shared = Arc<Mutex<Fake>>;

fn owned(value: impl Into<Value<'static>>) -> OwnedValue {
    OwnedValue::try_from(value.into()).unwrap()
}
fn path(value: &'static str) -> OwnedValue {
    owned(ObjectPath::try_from(value).unwrap())
}
fn add(objects: &mut ManagedObjects, at: &str, interface: &str, properties: &[(&str, OwnedValue)]) {
    objects
        .entry(OwnedObjectPath::try_from(at).unwrap())
        .or_default()
        .insert(
            interface.try_into().unwrap(),
            properties
                .iter()
                .map(|(key, value)| ((*key).into(), value.try_clone().unwrap()))
                .collect(),
        );
}
struct Manager(Shared);

#[zbus::interface(name = "org.freedesktop.DBus.ObjectManager")]
impl Manager {
    fn get_managed_objects(&self) -> ManagedObjects {
        let state = self.0.lock().unwrap();
        let mut objects = HashMap::new();
        add(
            &mut objects,
            SATA,
            DRIVE,
            &[
                ("ConnectionBus", owned("ata")),
                ("TimeDetected", owned(10_u64)),
            ],
        );
        add(
            &mut objects,
            OS_BLOCK,
            BLOCK,
            &[("Drive", path(SATA)), ("HintSystem", owned(true))],
        );
        add(
            &mut objects,
            OS_BLOCK,
            FS,
            &[("MountPoints", owned(vec![b"/\0".to_vec()]))],
        );
        if state.removed {
            return objects;
        }
        add(
            &mut objects,
            USB,
            DRIVE,
            &[
                ("ConnectionBus", owned("usb")),
                ("TimeDetected", owned(20_u64)),
                ("Vendor", owned("Fixture")),
                ("Model", owned("USB")),
                ("CanPowerOff", owned(true)),
                ("Ejectable", owned(false)),
            ],
        );
        add(
            &mut objects,
            USB_BLOCK,
            BLOCK,
            &[
                ("Drive", path(USB)),
                ("DeviceNumber", owned(100_u64)),
                ("HintSystem", owned(false)),
                ("HintIgnore", owned(false)),
                ("IdLabel", owned("Encrypted fixture")),
                ("IdUUID", owned("fixture-uuid")),
            ],
        );
        add(&mut objects, USB_BLOCK, CRYPT, &[]);
        if state.unlocked {
            add(
                &mut objects,
                MAPPING,
                BLOCK,
                &[("CryptoBackingDevice", path(USB_BLOCK))],
            );
            let mounts = if state.mounted {
                vec![b"/run/media/test/fixture\0".to_vec()]
            } else {
                vec![]
            };
            add(&mut objects, MAPPING, FS, &[("MountPoints", owned(mounts))]);
        }
        objects
    }
}

struct Encrypted(Shared);

#[zbus::interface(name = "org.freedesktop.UDisks2.Encrypted")]
impl Encrypted {
    fn unlock(&self, passphrase: &str, options: HashMap<String, OwnedValue>) -> OwnedObjectPath {
        assert_eq!(passphrase, "fixture-passphrase");
        assert!(options.is_empty());
        let mut state = self.0.lock().unwrap();
        state.calls.push("unlock");
        state.unlocked = true;
        MAPPING.try_into().unwrap()
    }
    fn lock(&self, options: HashMap<String, OwnedValue>) {
        assert!(options.is_empty());
        let mut state = self.0.lock().unwrap();
        assert!(!state.mounted);
        state.calls.push("lock");
        state.unlocked = false;
    }
}
struct Filesystem(Shared);

#[zbus::interface(name = "org.freedesktop.UDisks2.Filesystem")]
impl Filesystem {
    fn mount(&self, options: HashMap<String, OwnedValue>) -> &'static str {
        assert!(options.is_empty());
        let mut state = self.0.lock().unwrap();
        state.calls.push("mount");
        state.mounted = true;
        "/run/media/test/fixture"
    }
    fn unmount(&self, options: HashMap<String, OwnedValue>) -> zbus::fdo::Result<()> {
        assert!(options.is_empty());
        let mut state = self.0.lock().unwrap();
        state.calls.push("unmount");
        if state.busy {
            return Err(Error::Failed("busy".into()));
        }
        state.mounted = false;
        Ok(())
    }
}

struct Drive(Shared);

#[zbus::interface(name = "org.freedesktop.UDisks2.Drive")]
impl Drive {
    fn power_off(&self, options: HashMap<String, OwnedValue>) {
        assert!(options.is_empty());
        let mut state = self.0.lock().unwrap();
        assert!(!state.mounted && !state.unlocked);
        state.calls.push("power_off");
        state.removed = true;
    }
}

#[tokio::test]
async fn removable_only_and_busy_eject_never_locks_or_powers_off_a_mounted_drive() {
    let bus = Bus::new().await;
    let data = Arc::new(Mutex::new(Fake::default()));
    let _server = zbus::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name(SERVICE)
        .unwrap()
        .serve_at(ROOT, Manager(data.clone()))
        .unwrap()
        .serve_at(USB, Drive(data.clone()))
        .unwrap()
        .serve_at(USB_BLOCK, Encrypted(data.clone()))
        .unwrap()
        .serve_at(MAPPING, Filesystem(data.clone()))
        .unwrap()
        .build()
        .await
        .unwrap();
    let connection = bus.connect().await;
    let status = read(&connection).await.unwrap();
    assert_eq!(status.drives.len(), 1);
    let drive = &status.drives[0];
    let id = drive.volumes[0].id.clone();
    assert!(drive.volumes[0].locked && !drive.volumes[0].filesystem);
    assert!(apply(
        &connection,
        Request::Mount {
            id: OS_BLOCK.into()
        }
    )
    .await
    .is_err());
    let request = Request::Unlock {
        id: id.clone(),
        password: crate::services::connectivity::Secret::new("fixture-passphrase".into()),
    };
    assert!(!format!("{request:?}").contains("fixture-passphrase"));
    apply(&connection, request).await.unwrap();
    apply(&connection, Request::Mount { id }).await.unwrap();
    let eject = Request::Eject {
        id: drive.id.clone(),
    };
    data.lock().unwrap().busy = true;
    assert!(apply(&connection, eject.clone()).await.is_err());
    assert_eq!(data.lock().unwrap().calls, ["unlock", "mount", "unmount"]);
    data.lock().unwrap().busy = false;
    apply(&connection, eject.clone()).await.unwrap();
    assert_eq!(
        data.lock().unwrap().calls,
        ["unlock", "mount", "unmount", "unmount", "lock", "power_off"]
    );
    assert!(apply(&connection, eject).await.is_err());
}
