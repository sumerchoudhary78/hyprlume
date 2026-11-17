use super::super::{Drive, Volume};
use std::{
    collections::HashMap,
    ffi::OsString,
    os::unix::ffi::OsStringExt,
    path::{Component, Path, PathBuf},
};
use zbus::{
    fdo::ManagedObjects,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue},
};

const DRIVE: &str = "org.freedesktop.UDisks2.Drive";
const BLOCK: &str = "org.freedesktop.UDisks2.Block";
pub(super) const FS: &str = "org.freedesktop.UDisks2.Filesystem";
pub(super) const CRYPT: &str = "org.freedesktop.UDisks2.Encrypted";

type Properties = HashMap<String, OwnedValue>;
fn interface<'a>(
    objects: &'a ManagedObjects,
    path: &OwnedObjectPath,
    name: &str,
) -> Option<&'a Properties> {
    objects
        .get(path)?
        .iter()
        .find(|(key, _)| key.as_str() == name)
        .map(|(_, value)| value)
}
fn text<'a>(props: &'a Properties, key: &str) -> &'a str {
    props
        .get(key)
        .and_then(|v| <&str>::try_from(v).ok())
        .unwrap_or("")
}
fn flag(props: &Properties, key: &str) -> Option<bool> {
    props.get(key).and_then(|v| bool::try_from(v).ok())
}
fn number(props: &Properties, key: &str) -> Option<u64> {
    props.get(key).and_then(|v| u64::try_from(v).ok())
}
fn object(props: &Properties, key: &str) -> Option<OwnedObjectPath> {
    let path = <&ObjectPath>::try_from(props.get(key)?).ok()?;
    (path.as_str() != "/").then(|| path.to_owned().into())
}
fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(96)
        .collect::<String>()
        .trim()
        .to_owned()
}
fn mount_path(raw: Vec<u8>) -> Option<PathBuf> {
    let bytes = raw.strip_suffix(&[0])?;
    if bytes.is_empty() || bytes.len() > 4096 || bytes.contains(&0) {
        return None;
    }
    let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));
    path.is_absolute().then_some(path)
}
fn safe_mount(path: &Path) -> bool {
    path.components()
        .all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
        && ["/run/media", "/media", "/mnt"]
            .iter()
            .any(|base| path.starts_with(base) && path != Path::new(base))
}

#[derive(Clone)]
pub(super) struct Device {
    pub(super) path: OwnedObjectPath,
    pub(super) row: Drive,
    pub(super) volumes: Vec<Entry>,
    pub(super) removal: Option<&'static str>,
    pub(super) protected: bool,
}
#[derive(Clone)]
pub(super) struct Entry {
    pub(super) row: Volume,
    pub(super) block: OwnedObjectPath,
    pub(super) filesystem: Option<OwnedObjectPath>,
    pub(super) mounts: Vec<PathBuf>,
}
pub(super) fn scan(objects: &ManagedObjects, owner: &str) -> Vec<Device> {
    let mut drives = Vec::new();
    for path in objects.keys() {
        let Some(props) = interface(objects, path, DRIVE) else {
            continue;
        };
        if text(props, "ConnectionBus") != "usb" && flag(props, "Removable") != Some(true) {
            continue;
        }
        let Some(detected) = number(props, "TimeDetected").filter(|stamp| *stamp != 0) else {
            continue;
        };
        let id = format!("{owner}:{path}:{detected}");
        let sibling = text(props, "SiblingId");
        let has_siblings = !sibling.is_empty()
            && objects.keys().any(|other| {
                path != other
                    && interface(objects, other, DRIVE)
                        .is_some_and(|props| text(props, "SiblingId") == sibling)
            });
        let removal = if flag(props, "Ejectable") == Some(true) {
            Some("Eject")
        } else if flag(props, "CanPowerOff") == Some(true) && !has_siblings {
            Some("PowerOff")
        } else {
            None
        };
        let mut drive = Device {
            path: path.clone(),
            row: Drive {
                id: id.clone(),
                label: clean(&format!(
                    "{} {}",
                    text(props, "Vendor"),
                    text(props, "Model")
                )),
                volumes: vec![],
                can_eject: false,
            },
            volumes: vec![],
            removal,
            protected: false,
        };
        if drive.row.label.is_empty() {
            drive.row.label = "Removable drive".into();
        }
        for block in objects.keys() {
            let Some(props) = interface(objects, block, BLOCK) else {
                continue;
            };
            if object(props, "Drive").as_ref() != Some(path)
                || object(props, "CryptoBackingDevice").is_some()
            {
                continue;
            }
            if flag(props, "HintSystem") != Some(false) || flag(props, "HintIgnore") != Some(false)
            {
                drive.protected = true;
                continue;
            }
            let Some(number) = number(props, "DeviceNumber") else {
                drive.protected = true;
                continue;
            };
            let encrypted = interface(objects, block, CRYPT).is_some();
            let cleartext = encrypted
                .then(|| {
                    objects.keys().find(|mapping| {
                        interface(objects, mapping, BLOCK)
                            .and_then(|props| object(props, "CryptoBackingDevice"))
                            .as_ref()
                            == Some(block)
                    })
                })
                .flatten();
            let filesystem = if encrypted { cleartext } else { Some(block) }
                .filter(|path| interface(objects, path, FS).is_some());
            if !encrypted && filesystem.is_none() {
                continue;
            }
            let mut malformed = false;
            let mut mounts = Vec::new();
            if let Some(path) = filesystem {
                let raw = interface(objects, path, FS)
                    .and_then(|props| props.get("MountPoints"))
                    .and_then(|value| value.try_clone().ok())
                    .and_then(|value| Vec::<Vec<u8>>::try_from(value).ok());
                match raw {
                    Some(paths) => {
                        for bytes in paths {
                            match mount_path(bytes) {
                                Some(path) => mounts.push(path),
                                None => malformed = true,
                            }
                        }
                    }
                    None => malformed = true,
                }
            }
            let protected = malformed || mounts.iter().any(|path| !safe_mount(path));
            drive.protected |= protected;
            let label = clean(text(props, "IdLabel"));
            let row = Volume {
                id: format!("{id}:{block}:{number}:{}", text(props, "IdUUID")),
                label: if label.is_empty() {
                    clean(block.as_str().rsplit('/').next().unwrap_or("Volume"))
                } else {
                    label
                },
                filesystem: filesystem.is_some(),
                mounted: !mounts.is_empty(),
                encrypted,
                locked: encrypted && cleartext.is_none(),
                protected,
            };
            drive.volumes.push(Entry {
                row,
                block: block.clone(),
                filesystem: filesystem.cloned(),
                mounts,
            });
        }
        drive
            .volumes
            .sort_by(|left, right| left.block.cmp(&right.block));
        drive.protected |= drive.volumes.len() > 8;
        drive.row.can_eject = !drive.protected && removal.is_some();
        drive.row.volumes = drive
            .volumes
            .iter()
            .take(8)
            .map(|entry| entry.row.clone())
            .collect();
        drives.push(drive);
    }
    drives.sort_by(|left, right| left.path.cmp(&right.path));
    drives
}
