//! A private fail-closed episode marker survives daemon/shell restarts.
use anyhow::{ensure, Context, Result};
use std::{
    io::Write,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
};
pub(super) fn path() -> Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR is required")?;
    Ok(PathBuf::from(runtime).join("hyprlume-lock/episode"))
}
pub(in crate::services) fn exists() -> bool {
    path().is_ok_and(|path| std::fs::symlink_metadata(path).is_ok())
}
pub(super) fn begin() -> Result<()> {
    let path = path()?;
    let parent = path.parent().unwrap();
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)?;
    let metadata = std::fs::symlink_metadata(parent)?;
    ensure!(
        metadata.is_dir()
            && !metadata.file_type().is_symlink()
            && metadata.uid() == rustix::process::geteuid().as_raw()
            && metadata.permissions().mode() & 0o077 == 0,
        "Lock runtime directory must be private and owned by this user"
    );
    if path.exists() {
        let metadata = std::fs::symlink_metadata(&path)?;
        ensure!(
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == rustix::process::geteuid().as_raw()
                && metadata.permissions().mode() & 0o077 == 0,
            "Unsafe lock episode marker"
        );
    } else {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(b"Authentication required\n")?;
        file.sync_all()?;
    }
    Ok(())
}
pub(super) fn completed() -> Result<()> {
    std::fs::remove_file(path()?).context("Clear completed lock episode")
}
