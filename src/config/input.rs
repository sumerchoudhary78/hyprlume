use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    pub layouts: String,
    pub variants: String,
    pub repeat_rate: u16,
    pub repeat_delay: u16,
    pub pointer_speed: f32,
    pub left_handed: bool,
    pub mouse_natural_scroll: bool,
    pub touchpad_natural_scroll: bool,
    pub tap_to_click: bool,
    pub disable_while_typing: bool,
    pub shortcut_bindings: bool,
    pub system_key_bindings: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            layouts: "us".into(),
            variants: String::new(),
            repeat_rate: 25,
            repeat_delay: 600,
            pointer_speed: 0.0,
            left_handed: false,
            mouse_natural_scroll: false,
            touchpad_natural_scroll: false,
            tap_to_click: true,
            disable_while_typing: true,
            shortcut_bindings: false,
            system_key_bindings: false,
        }
    }
}
impl Input {
    pub fn validate(&self) -> Result<()> {
        let layouts: Vec<_> = self.layouts.split(',').collect();
        let variants: Vec<_> = self.variants.split(',').collect();
        let token = |value: &str| {
            value.len() <= 32
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        };
        ensure!(
            layouts.len() <= 4
                && layouts
                    .iter()
                    .all(|layout| !layout.is_empty() && token(layout)),
            "Use one to four XKB layout IDs separated by commas"
        );
        ensure!(
            variants.iter().all(|variant| token(variant))
                && (variants.len() == 1 || variants.len() == layouts.len()),
            "Use one variant or a comma-separated variant for each layout"
        );
        ensure!(
            self.repeat_rate <= 200 && self.repeat_delay <= 2000,
            "Keyboard repeat must be 0–200 Hz with a delay of 0–2000 ms"
        );
        ensure!(
            self.pointer_speed.is_finite() && (-1.0..=1.0).contains(&self.pointer_speed),
            "Pointer speed must be between −1 and 1"
        );
        Ok(())
    }

    pub(crate) fn validate_catalog(&self, catalog: &str) -> Result<()> {
        self.validate()?;
        let mut section = "";
        let mut layouts = std::collections::HashSet::new();
        let mut variants = std::collections::HashSet::new();
        for line in catalog.lines().map(str::trim) {
            if let Some(next) = line.strip_prefix("! ") {
                section = next.trim();
                continue;
            }
            let mut words = line.split_whitespace();
            match (section, words.next(), words.next()) {
                ("layout", Some(layout), _) => {
                    layouts.insert(layout);
                }
                ("variant", Some(variant), Some(layout)) => {
                    if let Some(layout) = layout.strip_suffix(':') {
                        variants.insert((layout, variant));
                    }
                }
                _ => (),
            }
        }
        let selected: Vec<_> = self.variants.split(',').collect();
        for (index, layout) in self.layouts.split(',').enumerate() {
            ensure!(layouts.contains(layout), "Unknown XKB layout {layout}");
            let variant = selected[if selected.len() == 1 { 0 } else { index }];
            ensure!(
                variant.is_empty() || variants.contains(&(layout, variant)),
                "Unknown XKB variant {variant} for {layout}"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_validation_rejects_unknown_pairs_and_compositor_syntax() {
        let catalog = "! model\n  pc105 PC\n! layout\n  us English\n  gb British\n! variant\n  intl us: English International\n";
        let mut input = Input {
            layouts: "us,gb".into(),
            variants: "intl,".into(),
            ..Input::default()
        };
        input.validate_catalog(catalog).unwrap();
        input.variants = ",intl".into();
        assert!(input.validate_catalog(catalog).is_err());
        input.layouts = "us;exec".into();
        assert!(input.validate().is_err());
    }
}
