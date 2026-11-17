use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub appearance: Appearance,
    pub shortcuts: Shortcuts,
    pub services: Services,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Appearance {
    pub reduced_effects: bool,
    pub accent: Accent,
    pub scale: f32,
    pub refraction: f32,
    pub metrics: bool,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            reduced_effects: false,
            accent: Accent::Cyan,
            scale: 1.0,
            refraction: 16.0,
            metrics: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    #[default]
    Cyan,
    Violet,
    Mint,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Shortcuts {
    pub launcher: String,
    pub windows: String,
    pub tools: String,
    pub lock: String,
}
impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            launcher: "SUPER+SPACE".into(),
            windows: "SUPER+TAB".into(),
            tools: "SUPER+V".into(),
            lock: "SUPER+L".into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Services {
    pub clipboard_history: bool,
    pub notifications_dnd: bool,
}
impl Preferences {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.appearance.scale.is_finite() && (0.85..=1.2).contains(&self.appearance.scale),
            "UI scale must be between 0.85 and 1.2"
        );
        ensure!(
            self.appearance.refraction.is_finite()
                && (0.0..=24.0).contains(&self.appearance.refraction),
            "Glass refraction must be between 0 and 24"
        );
        let mut unique = std::collections::BTreeSet::new();
        for key in [
            &self.shortcuts.launcher,
            &self.shortcuts.windows,
            &self.shortcuts.tools,
            &self.shortcuts.lock,
        ] {
            validate_shortcut(key)?;
            let mut pieces: Vec<_> = key.split('+').collect();
            let key = pieces.pop().unwrap();
            pieces.sort_unstable();
            let normalized = format!("{}+{key}", pieces.join("+"));
            ensure!(
                unique.insert(normalized),
                "Shortcut combinations must be distinct"
            );
        }
        Ok(())
    }
}
pub fn validate_shortcut(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= 48,
        "A shortcut must contain modifiers and a key"
    );
    let parts: Vec<_> = value.split('+').collect();
    ensure!(
        (2..=5).contains(&parts.len()),
        "Use shortcuts such as SUPER+SPACE or SUPER+SHIFT+L"
    );
    let mut modifiers = std::collections::BTreeSet::new();
    for modifier in &parts[..parts.len() - 1] {
        ensure!(
            ["SUPER", "CTRL", "ALT", "SHIFT"].contains(modifier) && modifiers.insert(*modifier),
            "Shortcut modifiers must be unique SUPER, CTRL, ALT or SHIFT"
        );
    }
    let key = parts[parts.len() - 1];
    ensure!(
        (key.len() == 1
            && key
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()))
            || ["SPACE", "TAB", "RETURN", "ESCAPE", "HOME", "END", "UP", "DOWN", "LEFT", "RIGHT"]
                .contains(&key)
            || key
                .strip_prefix('F')
                .and_then(|n| n.parse::<u8>().ok())
                .is_some_and(|n| (1..=24).contains(&n) && key == format!("F{n}")),
        "Unsupported shortcut key"
    );
    Ok(())
}
