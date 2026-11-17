use super::{text, Native, LEGACY_PROFILES, PROFILES};
use crate::services::power::{types::Binding, types::Profiles, Hardware, Profile, Request};
use anyhow::{ensure, Result};
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

fn profile(name: &str) -> Option<Profile> {
    match name {
        "power-saver" => Some(Profile::PowerSaver),
        "balanced" => Some(Profile::Balanced),
        "performance" => Some(Profile::Performance),
        _ => None,
    }
}
async fn binding(native: &Native) -> Result<Binding> {
    match native
        .binding(PROFILES, "/org/freedesktop/UPower/PowerProfiles", PROFILES)
        .await
    {
        Ok(binding) => Ok(binding),
        Err(_) => {
            native
                .binding(
                    LEGACY_PROFILES,
                    "/net/hadess/PowerProfiles",
                    LEGACY_PROFILES,
                )
                .await
        }
    }
}
async fn snapshot(native: &Native) -> Result<Profiles> {
    let binding = binding(native).await?;
    let props = native.properties(&binding).await?;
    let entries: Vec<HashMap<String, OwnedValue>> = props
        .get("Profiles")
        .ok_or_else(|| anyhow::anyhow!("Missing power profiles"))?
        .try_clone()?
        .try_into()?;
    let mut choices = Vec::new();
    for entry in entries.iter().take(8) {
        if let Some(profile) = profile(&text(entry, "Profile", 32)) {
            if !choices.contains(&profile) {
                choices.push(profile);
            }
        }
    }
    Ok(Profiles {
        available: true,
        active: profile(&text(&props, "ActiveProfile", 32)),
        choices,
        degraded: text(&props, "PerformanceDegraded", 96),
        error: String::new(),
        binding: Some(binding),
    })
}
pub(super) async fn read(native: &Native) -> Profiles {
    snapshot(native).await.unwrap_or_else(|_| Profiles {
        error: "Power profiles daemon is unavailable".into(),
        ..Profiles::default()
    })
}
pub(super) async fn set(native: &Native, observed: Hardware, profile: Profile) -> Result<String> {
    observed.validate(&Request::Profile { profile })?;
    let current = snapshot(native).await?;
    ensure!(
        current.binding == observed.profiles.binding && current.choices.contains(&profile),
        "Power profile service changed; try again"
    );
    let binding = current.binding.unwrap();
    native
        .proxy(&binding)
        .await?
        .set_property("ActiveProfile", profile.name())
        .await?;
    ensure!(
        snapshot(native).await?.active == Some(profile),
        "Power profiles daemon did not confirm the selected profile"
    );
    Ok(format!("Power profile: {}", profile.name()))
}
