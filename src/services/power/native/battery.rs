use super::{flag, Native, UPOWER};
use crate::services::power::Battery;
use anyhow::Result;

async fn snapshot(native: &Native) -> Result<Battery> {
    let binding = native
        .binding(
            UPOWER,
            "/org/freedesktop/UPower/devices/DisplayDevice",
            "org.freedesktop.UPower.Device",
        )
        .await?;
    let props = native.properties(&binding).await?;
    let present = flag(&props, "IsPresent");
    let state = props
        .get("State")
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0);
    let state_name = match state {
        1 => "charging",
        2 => "discharging",
        3 => "empty",
        4 => "full",
        5 => "pending_charge",
        6 => "pending_discharge",
        _ => "unknown",
    };
    let percentage = props
        .get("Percentage")
        .and_then(|value| f64::try_from(value).ok())
        .filter(|number| number.is_finite() && (0.0..=100.0).contains(number) && present);
    let seconds = match state {
        1 => "TimeToFull",
        2 => "TimeToEmpty",
        _ => "",
    };
    let seconds_remaining = props
        .get(seconds)
        .and_then(|value| i64::try_from(value).ok())
        .filter(|seconds| *seconds > 0 && *seconds <= 7 * 24 * 60 * 60)
        .map(|seconds| seconds as u64);
    Ok(Battery {
        available: true,
        present,
        percentage,
        state: state_name.into(),
        seconds_remaining,
        error: String::new(),
    })
}
pub(super) async fn read(native: &Native) -> Battery {
    snapshot(native).await.unwrap_or_else(|_| Battery {
        error: "UPower battery information is unavailable".into(),
        ..Battery::default()
    })
}
