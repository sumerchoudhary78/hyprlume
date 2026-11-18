use anyhow::{ensure, Context, Result};
use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
pub(crate) fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            ensure!(
                metadata.is_file()
                    && !metadata.file_type().is_symlink()
                    && metadata.uid() == rustix::process::geteuid().as_raw(),
                "Settings file must be a regular file owned by this user"
            );
            ensure!(
                metadata.len() <= 256 * 1024,
                "Settings file exceeds 256 KiB"
            );
            Ok(Some(std::fs::read(path)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
pub(crate) fn save(path: &Path, expected: Option<&[u8]>, contents: &[u8]) -> Result<()> {
    ensure!(
        read(path)?.as_deref() == expected,
        "Configuration was edited outside this view; reload before saving"
    );
    let parent = path
        .parent()
        .context("Settings path has no parent directory")?;
    std::fs::create_dir_all(parent)?;
    let parent_metadata = std::fs::metadata(parent)?;
    ensure!(
        parent_metadata.is_dir()
            && parent_metadata.uid() == rustix::process::geteuid().as_raw()
            && parent_metadata.permissions().mode() & 0o022 == 0,
        "Settings directory must be owned and not writable by other users"
    );
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = parent.join(format!(".hyprlume-{}-{stamp}.tmp", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        // Verify again after flushing, then replace atomically in the same directory.
        ensure!(
            read(path)?.as_deref() == expected,
            "Configuration changed while saving; reload before saving"
        );
        std::fs::rename(&temporary, path)?;
        std::fs::File::open(parent)?.sync_all()?;
        ensure!(
            std::fs::metadata(path)?.permissions().mode() & 0o077 == 0,
            "Settings file permissions are not private"
        );
        Ok::<_, anyhow::Error>(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
