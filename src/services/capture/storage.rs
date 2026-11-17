//! Exclusive, private output storage, shared by screenshots and recordings.
use anyhow::{ensure, Context, Result};
use std::{
    fs::{File, OpenOptions},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) fn output_file(record: bool) -> Result<(PathBuf, File)> {
    let home = std::env::var_os("HOME").context("HOME is required for capture storage")?;
    let variable = if record {
        "XDG_VIDEOS_DIR"
    } else {
        "XDG_PICTURES_DIR"
    };
    let base = std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| PathBuf::from(home).join(if record { "Videos" } else { "Pictures" }));
    let directory = base.join("Hyprlume");
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&directory)?;
    let metadata = std::fs::symlink_metadata(&directory)?;
    ensure!(
        metadata.is_dir()
            && !metadata.file_type().is_symlink()
            && metadata.uid() == rustix::process::geteuid().as_raw()
            && metadata.permissions().mode() & 0o077 == 0,
        "Capture directory must be an owned private directory (0700)"
    );
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = directory.join(format!(
        "capture-{}-{stamp}.{}",
        std::process::id(),
        if record { "mkv" } else { "png" }
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    Ok((path, file))
}
