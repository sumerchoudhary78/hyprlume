use super::{session, Backend, Native};
use crate::services::power::{Backlight, Hardware, Request};
use anyhow::{ensure, Context, Result};
use std::{fs, io::Read, os::unix::fs::MetadataExt, path::Path};

fn number(path: &Path) -> Result<u32> {
    let mut text = String::new();
    fs::File::open(path)?.take(64).read_to_string(&mut text)?;
    Ok(text.trim().parse()?)
}
pub(super) fn scan(root: &Path) -> Result<Vec<Backlight>> {
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(Vec::new());
    };
    let mut lights = Vec::new();
    for entry in entries.take(32).flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty()
            || name.len() > 128
            || matches!(name.as_str(), "." | "..")
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c))
        {
            continue;
        }
        let path = entry.path();
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        let (Ok(max), Ok(value)) = (
            number(&path.join("max_brightness")),
            number(&path.join("brightness")),
        ) else {
            continue;
        };
        if max == 0 || value > max {
            continue;
        }
        lights.push(Backlight {
            id: format!(
                "{}:{}:{}:{}:{}:{}",
                name,
                meta.dev(),
                meta.ino(),
                meta.ctime(),
                meta.ctime_nsec(),
                max
            ),
            name,
            percent: ((u64::from(value) * 100 + u64::from(max) / 2) / u64::from(max)) as u8,
            value,
            max,
        });
    }
    lights.sort_by(|a, b| a.name.cmp(&b.name));
    lights.truncate(8);
    Ok(lights)
}
pub(super) async fn set(
    native: &Native,
    observed: Hardware,
    id: &str,
    percent: u8,
) -> Result<String> {
    observed.validate(&Request::Brightness {
        device: id.into(),
        percent,
    })?;
    let target = observed
        .session
        .target
        .context("No graphical session for brightness")?;
    session::verify(native, &target).await?;
    let lights = native.backlights().await?;
    let light = lights
        .iter()
        .find(|light| light.id == id)
        .context("Backlight changed or disappeared")?;
    let value = ((u64::from(light.max) * u64::from(percent) + 50) / 100).max(1) as u32;
    native
        .proxy(&target.binding)
        .await?
        .call::<_, _, ()>("SetBrightness", &("backlight", light.name.as_str(), value))
        .await?;
    let actual = native.backlights().await?;
    let confirmed = actual
        .iter()
        .find(|light| light.id == id && light.value == value)
        .context("Driver did not confirm the requested brightness")?;
    Ok(format!("Brightness {}%", confirmed.percent))
}

pub(super) async fn restore(
    native: &Native,
    observed: Hardware,
    id: &str,
    expected: u32,
    value: u32,
) -> Result<String> {
    let target = observed
        .session
        .target
        .context("No graphical session for brightness restoration")?;
    session::verify(native, &target).await?;
    let lights = native.backlights().await?;
    let light = lights
        .iter()
        .find(|light| light.id == id)
        .context("Backlight changed or disappeared")?;
    ensure!(value > 0 && value <= light.max, "Invalid saved brightness");
    if light.value != expected {
        return Ok("Brightness was adjusted independently; leaving it unchanged".into());
    }
    native
        .proxy(&target.binding)
        .await?
        .call::<_, _, ()>("SetBrightness", &("backlight", light.name.as_str(), value))
        .await?;
    ensure!(
        native
            .backlights()
            .await?
            .iter()
            .any(|light| light.id == id && light.value == value),
        "Driver did not confirm restored brightness"
    );
    Ok("Previous brightness restored".into())
}
