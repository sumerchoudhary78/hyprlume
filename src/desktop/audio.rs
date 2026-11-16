//! PipeWire volume/mute actions, checked against the audio server.
use super::run;
use anyhow::{bail, Context, Result};

pub(super) async fn set_volume(percent: u8) -> Result<String> {
    run(
        "wpctl",
        &["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{percent}%")],
    )
    .await?;
    let actual = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]).await?;
    let volume: f32 = actual
        .split_whitespace()
        .nth(1)
        .context("wpctl returned no volume")?
        .parse()?;
    if (volume * 100.0 - percent as f32).abs() > 1.1 {
        bail!("audio server did not confirm the requested volume");
    }
    Ok(format!("Volume {percent}% confirmed"))
}

pub(super) async fn set_mute(muted: bool) -> Result<String> {
    run(
        "wpctl",
        &[
            "set-mute",
            "@DEFAULT_AUDIO_SINK@",
            if muted { "1" } else { "0" },
        ],
    )
    .await?;
    let actual = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]).await?;
    if actual.contains("[MUTED]") != muted {
        bail!("audio server did not confirm mute state");
    }
    Ok(if muted {
        "Audio muted"
    } else {
        "Audio unmuted"
    }
    .into())
}
