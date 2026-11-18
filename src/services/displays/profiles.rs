//! Only confirmed layouts are written. Connector and display identity must all match.
use super::layout::{snapshot, Layout, Output, Plan};
use crate::services::settings::storage;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Profiles {
    version: u16,
    layouts: Vec<Layout>,
}

fn decode(source: Option<&[u8]>) -> Result<Profiles> {
    let profiles = if let Some(bytes) = source {
        serde_json::from_slice(bytes)?
    } else {
        Profiles {
            version: 1,
            layouts: vec![],
        }
    };
    ensure!(
        profiles.version == 1 && profiles.layouts.len() <= 8,
        "Unsupported display profiles file"
    );
    Ok(profiles)
}

pub(super) fn remember(path: &Path, layout: Layout) -> Result<()> {
    let existing = storage::read(path)?;
    let mut profiles = decode(existing.as_deref())?;
    profiles
        .layouts
        .retain(|profile| profile.topology != layout.topology);
    profiles.layouts.insert(0, layout);
    profiles.layouts.truncate(8);
    let contents = serde_json::to_vec_pretty(&profiles)?;
    ensure!(
        contents.len() <= 256 * 1024,
        "Display profiles file is too large"
    );
    storage::save(path, existing.as_deref(), &contents)
}

pub(super) fn plan(path: &Path, outputs: &[Output]) -> Result<Option<Plan>> {
    let existing = storage::read(path)?;
    let current = snapshot(outputs);
    let profiles = decode(existing.as_deref())?;
    let Some(profile) = profiles
        .layouts
        .into_iter()
        .find(|profile| profile.topology == current.topology)
    else {
        return Ok(None);
    };
    if !profile
        .settings
        .iter()
        .map(|s| &s.name)
        .eq(current.settings.iter().map(|s| &s.name))
    {
        return Ok(None);
    }
    let plan = Plan::new(outputs, profile.settings)?;
    Ok((!plan.changes().is_empty()).then_some(plan))
}
