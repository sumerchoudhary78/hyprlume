use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Identity {
    name: String,
    description: Option<String>,
    make: Option<String>,
    model: Option<String>,
    serial: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
struct Position {
    x: i32,
    y: i32,
}
#[derive(Deserialize)]
struct RawMode {
    width: u32,
    height: u32,
    refresh: f64,
    current: bool,
}
#[derive(Deserialize)]
struct RawOutput {
    #[serde(flatten)]
    identity: Identity,
    enabled: bool,
    modes: Vec<RawMode>,
    #[serde(default)]
    position: Position,
    #[serde(default)]
    scale: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Output {
    #[serde(skip)]
    identity: Identity,
    pub name: String,
    label: String,
    pub modes: Vec<String>,
    pub active: Option<Setting>,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    pub name: String,
    pub mode: String,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Layout {
    pub topology: Vec<Identity>,
    pub settings: Vec<Setting>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Plan {
    pub before: Layout,
    pub after: Layout,
}

fn observe_mode(mode: &RawMode) -> Option<String> {
    if (1..=32768).contains(&mode.width)
        && (1..=32768).contains(&mode.height)
        && mode.refresh.is_finite()
        && (0.0..=1000.0).contains(&mode.refresh)
    {
        // wlr-randr prints an f32, but modes are reported in integral millihertz.
        let rate = (mode.refresh * 1000.0).round() as u32;
        return Some(format!(
            "{}x{}@{}.{:03}Hz",
            mode.width,
            mode.height,
            rate / 1000,
            rate % 1000
        ));
    }
    None
}

pub(super) fn parse(text: &str) -> Result<Vec<Output>> {
    let mut raw: Vec<RawOutput> =
        serde_json::from_str(text).context("Read the compositor's outputs")?;
    ensure!(raw.len() <= 12, "More than 12 connected outputs");
    raw.sort_by(|a, b| a.identity.name.cmp(&b.identity.name));
    ensure!(
        raw.windows(2)
            .all(|pair| pair[0].identity.name != pair[1].identity.name),
        "Duplicate output names"
    );
    raw.into_iter()
        .map(|output| {
            let name = &output.identity.name;
            ensure!(
                !name.is_empty()
                    && name.len() <= 64
                    && !name.starts_with('-')
                    && name
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c)),
                "Unsupported output name"
            );
            let mode = output
                .modes
                .iter()
                .find(|mode| mode.current)
                .and_then(observe_mode);
            let modes: Vec<_> = output
                .modes
                .iter()
                .filter_map(observe_mode)
                .take(128)
                .collect();
            let active = if output.enabled {
                Some(Setting {
                    name: name.clone(),
                    mode: mode.context("An enabled output did not report its active mode")?,
                    x: output.position.x,
                    y: output.position.y,
                    scale: output.scale,
                })
            } else {
                None
            };
            let label = output
                .identity
                .description
                .as_deref()
                .unwrap_or(name)
                .chars()
                .filter(|c| !c.is_control())
                .take(128)
                .collect();
            Ok(Output {
                name: name.clone(),
                label,
                modes,
                active,
                identity: output.identity,
            })
        })
        .collect()
}

pub(super) fn snapshot(outputs: &[Output]) -> Layout {
    Layout {
        topology: outputs
            .iter()
            .map(|output| output.identity.clone())
            .collect(),
        settings: outputs
            .iter()
            .filter_map(|output| output.active.clone())
            .collect(),
    }
}

impl Plan {
    pub(super) fn new(outputs: &[Output], settings: Vec<Setting>) -> Result<Self> {
        let before = snapshot(outputs);
        let mut after = before.clone();
        ensure!(
            !settings.is_empty() && settings.len() <= after.settings.len(),
            "Select an active output"
        );
        let mut edited = std::collections::HashSet::new();
        for setting in settings {
            ensure!(
                edited.insert(setting.name.clone()),
                "Each output can be edited once"
            );
            let output = outputs
                .iter()
                .find(|o| o.name == setting.name)
                .context("The output disconnected")?;
            let current = after
                .settings
                .iter_mut()
                .find(|s| s.name == setting.name)
                .context("The output is disabled")?;
            ensure!(
                setting.mode == current.mode || output.modes.contains(&setting.mode),
                "Select a supported resolution and refresh rate"
            );
            ensure!(
                (-32768..=32768).contains(&setting.x) && (-32768..=32768).contains(&setting.y),
                "Output position is out of range"
            );
            ensure!(
                setting.scale.is_finite() && (0.5..=4.0).contains(&setting.scale),
                "Scale must be 0.5–4"
            );
            *current = setting;
        }
        Ok(Self { before, after })
    }

    pub(super) fn changes(&self) -> Vec<Setting> {
        self.after
            .settings
            .iter()
            .zip(&self.before.settings)
            .filter(|(next, previous)| !next.matches(previous))
            .map(|(next, _)| next.clone())
            .collect()
    }
    pub(super) fn restoration(&self, observed: &[Output]) -> Vec<Setting> {
        let current = snapshot(observed);
        let mut settings = current.settings.clone();
        for output in observed {
            if !self.before.topology.contains(&output.identity) {
                continue;
            }
            let Some(before) = self.before.settings.iter().find(|s| s.name == output.name) else {
                continue;
            };
            let Some(target) = settings.iter_mut().find(|s| s.name == output.name) else {
                continue;
            };
            if output.modes.contains(&before.mode) || target.mode == before.mode {
                *target = before.clone();
            }
        }
        // Hyprland can recalculate auto-positioned neighbors when an output
        // changes. Send all active outputs so untouched neighbors stay put.
        if settings
            .iter()
            .zip(&current.settings)
            .all(|(target, actual)| target.matches(actual))
        {
            return Vec::new();
        }
        settings
    }
}
impl Setting {
    pub(super) fn matches(&self, other: &Self) -> bool {
        self.name == other.name
            && self.mode == other.mode
            && self.x == other.x
            && self.y == other.y
            && (self.scale - other.scale).abs() <= 0.002
    }
}
impl Layout {
    pub(super) fn matches(&self, other: &Self) -> bool {
        self.topology == other.topology
            && self.settings.len() == other.settings.len()
            && self
                .settings
                .iter()
                .zip(&other.settings)
                .all(|(a, b)| a.matches(b))
    }
}
