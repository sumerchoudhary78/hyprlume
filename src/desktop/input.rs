//! Narrow Hyprland input operations and a generated user-owned include.
use super::{hyprland::uses_lua, Desktop};
use crate::config::{Input, Shortcuts};
use anyhow::{ensure, Context, Result};
use hyprpilot_core::Connection;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SystemKey {
    VolumeUp,
    VolumeDown,
    VolumeMute,
    MicMute,
    BrightnessUp,
    BrightnessDown,
    MediaPlayPause,
    MediaNext,
    MediaPrevious,
    MediaStop,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Status,
    Apply { revision: u64 },
    WriteProfile { revision: u64 },
    NextLayout,
}

pub(crate) async fn read_catalog(input: &Input) -> Result<()> {
    for path in [
        "/usr/share/X11/xkb/rules/evdev.lst",
        "/usr/share/xkeyboard-config-2/rules/evdev.lst",
    ] {
        match tokio::fs::read_to_string(path).await {
            Ok(catalog) => return input.validate_catalog(&catalog),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error).context("Read the installed XKB layout catalog"),
        }
    }
    anyhow::bail!("Install xkeyboard-config to validate keyboard layouts")
}

impl SystemKey {
    pub(crate) fn presentation(self) -> (&'static str, &'static str) {
        use SystemKey::*;
        match self {
            VolumeUp | VolumeDown | VolumeMute => ("Volume", "volume-high"),
            MicMute => ("Microphone", "microphone"),
            BrightnessUp | BrightnessDown => ("Brightness", "brightness"),
            MediaPlayPause | MediaNext | MediaPrevious | MediaStop => ("Media", "play"),
        }
    }

    pub(crate) async fn apply_audio(self) -> Result<(String, Option<u8>)> {
        use super::run;
        use SystemKey::*;
        const SINK: &str = "@DEFAULT_AUDIO_SINK@";
        const SOURCE: &str = "@DEFAULT_AUDIO_SOURCE@";
        let target = match self {
            VolumeUp | VolumeDown => {
                let value = if matches!(self, VolumeUp) {
                    "5%+"
                } else {
                    "5%-"
                };
                run("wpctl", &["set-volume", "-l", "1.0", SINK, value]).await?;
                SINK
            }
            VolumeMute | MicMute => {
                let target = if matches!(self, MicMute) {
                    SOURCE
                } else {
                    SINK
                };
                run("wpctl", &["set-mute", target, "toggle"]).await?;
                target
            }
            MediaPlayPause | MediaNext | MediaPrevious | MediaStop => {
                let (action, label) = match self {
                    MediaPlayPause => ("play-pause", "Playback toggled"),
                    MediaNext => ("next", "Next track"),
                    MediaPrevious => ("previous", "Previous track"),
                    MediaStop => ("stop", "Playback stopped"),
                    _ => unreachable!(),
                };
                run("playerctl", &[action]).await?;
                return Ok((label.into(), None));
            }
            BrightnessUp | BrightnessDown => unreachable!(),
        };
        let actual = run("wpctl", &["get-volume", target]).await?;
        if actual.contains("[MUTED]") {
            return Ok(("Muted".into(), Some(0)));
        }
        if matches!(self, MicMute) {
            return Ok(("Unmuted".into(), None));
        }
        let volume: f32 = actual
            .split_whitespace()
            .nth(1)
            .context("wpctl returned no volume")?
            .parse()?;
        ensure!(
            volume.is_finite() && volume >= 0.0,
            "Invalid audio server volume"
        );
        let percent = (volume * 100.0).round() as u16;
        Ok((format!("{percent}%"), Some(percent.min(100) as u8)))
    }
}

const SYSTEM_BINDINGS: [(&str, &str, bool); 10] = [
    ("XF86AudioRaiseVolume", "volume-up", true),
    ("XF86AudioLowerVolume", "volume-down", true),
    ("XF86AudioMute", "volume-mute", false),
    ("XF86AudioMicMute", "mic-mute", false),
    ("XF86MonBrightnessUp", "brightness-up", true),
    ("XF86MonBrightnessDown", "brightness-down", true),
    ("XF86AudioPlay", "media-play-pause", false),
    ("XF86AudioNext", "media-next", false),
    ("XF86AudioPrev", "media-previous", false),
    ("XF86AudioStop", "media-stop", false),
];

#[derive(Debug, Deserialize, Serialize)]
pub struct Keyboard {
    name: String,
    #[serde(default)]
    active_keymap: String,
    #[serde(default)]
    main: bool,
}
#[derive(Deserialize)]
struct Devices {
    keyboards: Vec<Keyboard>,
}

#[derive(Default, Serialize)]
pub struct Status {
    pub available: bool,
    pub keyboards: Vec<Keyboard>,
    pub profile: String,
    pub message: String,
}
#[derive(Clone, Copy)]
enum OptionValue<'a> {
    Text(&'a str),
    Number(f32),
    Integer(u16),
    Flag(bool),
}
impl OptionValue<'_> {
    fn render(self, lua: bool) -> String {
        match self {
            Self::Text(text) if lua => serde_json::to_string(text).expect("serialize option text"),
            Self::Text(text) => text.into(),
            Self::Number(number) => number.to_string(),
            Self::Integer(number) => number.to_string(),
            Self::Flag(flag) => flag.to_string(),
        }
    }
}
fn options(profile: &Input) -> [(&'static str, OptionValue<'_>); 10] {
    use OptionValue::*;
    [
        ("kb_layout", Text(&profile.layouts)),
        ("kb_variant", Text(&profile.variants)),
        ("repeat_rate", Integer(profile.repeat_rate)),
        ("repeat_delay", Integer(profile.repeat_delay)),
        ("sensitivity", Number(profile.pointer_speed)),
        ("left_handed", Flag(profile.left_handed)),
        ("natural_scroll", Flag(profile.mouse_natural_scroll)),
        (
            "touchpad:natural_scroll",
            Flag(profile.touchpad_natural_scroll),
        ),
        ("touchpad:tap-to-click", Flag(profile.tap_to_click)),
        (
            "touchpad:disable_while_typing",
            Flag(profile.disable_while_typing),
        ),
    ]
}
fn lua_options(profile: &Input) -> String {
    let fields: Vec<_> = options(profile)
        .into_iter()
        .map(|(key, value)| {
            format!(
                "[\"input.{}\"]={}",
                // Lua spells the legacy tap-to-click option with underscores.
                key.replace(':', ".").replace('-', "_"),
                value.render(true)
            )
        })
        .collect();
    format!("hl.config({{{}}})", fields.join(", "))
}
struct Binding {
    key: String,
    command: String,
    repeat: bool,
}
fn bindings(input: &Input, shortcuts: &Shortcuts) -> Vec<Binding> {
    let mut bindings = Vec::new();
    if input.shortcut_bindings {
        for (key, action) in [
            (&shortcuts.launcher, "launcher"),
            (&shortcuts.windows, "launcher --windows"),
            (&shortcuts.tools, "clipboard"),
            (&shortcuts.lock, "lock"),
        ] {
            bindings.push(Binding {
                key: key.clone(),
                command: format!("hyprlume {action}"),
                repeat: false,
            });
        }
    }
    if input.system_key_bindings {
        for (key, action, repeat) in SYSTEM_BINDINGS {
            bindings.push(Binding {
                key: key.into(),
                command: format!("hyprlume system-key {action}"),
                repeat,
            });
        }
    }
    bindings
}
fn split(binding: &str) -> (&str, &str) {
    binding.rsplit_once('+').unwrap_or(("", binding))
}
fn render(input: &Input, keys: &[Binding], lua: bool) -> String {
    let mut profile = String::new();
    if lua {
        writeln!(
            profile,
            "-- Generated by Hyprlume. Edit input preferences in Settings.\n{}",
            lua_options(input)
        )
        .unwrap();
    } else {
        writeln!(
            profile,
            "# Generated by Hyprlume. Edit input preferences in Settings."
        )
        .unwrap();
        for (key, value) in options(input) {
            writeln!(profile, "input:{key} = {}", value.render(false)).unwrap();
        }
    }
    for binding in keys {
        if lua {
            writeln!(
                profile,
                "hl.bind(\"{}\", hl.dsp.exec_cmd(\"{}\"), {{ repeating = {}, description = \"Hyprlume: {}\" }})",
                binding.key.replace('+', " + "),
                binding.command,
                binding.repeat,
                binding.command
            )
            .unwrap();
        } else {
            let (modifiers, key) = split(&binding.key);
            let flag = if binding.repeat { "e" } else { "" };
            writeln!(
                profile,
                "bind{flag} = {}, {key}, exec, {}",
                modifiers.replace('+', " "),
                binding.command
            )
            .unwrap();
        }
    }
    profile
}
#[derive(Deserialize)]
struct ExistingBind {
    key: String,
    modmask: u32,
    dispatcher: String,
    arg: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    submap: String,
}
fn check_conflicts(current: &[ExistingBind], desired: &[Binding]) -> Result<()> {
    for requested in desired {
        let (modifiers, key) = split(&requested.key);
        let mask = modifiers
            .split('+')
            .map(|modifier| match modifier {
                "SHIFT" => 1,
                "CTRL" => 4,
                "ALT" => 8,
                "SUPER" => 64,
                _ => 0,
            })
            .fold(0, |mask, value| mask | value);
        ensure!(
            !current.iter().any(|existing| {
                matches!(existing.submap.as_str(), "" | "default")
                    && existing.modmask == mask
                    && existing.key.eq_ignore_ascii_case(key)
                    && !(existing.dispatcher == "exec" && existing.arg.trim() == requested.command
                        || existing.description == format!("Hyprlume: {}", requested.command))
            }),
            "{} is already bound in the compositor",
            requested.key
        );
    }
    Ok(())
}
impl Desktop {
    fn input_connection(&self) -> Result<&Connection> {
        ensure!(!self.demo, "Input changes are disabled in demo mode");
        self.connection
            .as_ref()
            .context("No Hyprland compositor for this session")
    }

    pub(crate) async fn input_devices(&self) -> Result<Vec<Keyboard>> {
        let connection = self.input_connection()?;
        let mut devices: Devices = serde_json::from_str(&connection.send_raw("j/devices").await?)?;
        devices.keyboards.truncate(16);
        for keyboard in &mut devices.keyboards {
            keyboard.name = display_text(&keyboard.name);
            keyboard.active_keymap = display_text(&keyboard.active_keymap);
        }
        Ok(devices.keyboards)
    }

    pub(crate) async fn apply_input(&self, input: &Input) -> Result<()> {
        input.validate()?;
        let connection = self.input_connection()?;
        if uses_lua(connection).await? {
            let result = connection
                .send_raw(&format!("eval {}", lua_options(input)))
                .await?;
            ensure!(result.trim() == "ok", "Hyprland rejected the input options");
        } else {
            // The batch parser trims each command, including an empty option's
            // separator. Send that option separately so Hyprlang can clear it.
            if input.variants.is_empty() {
                let result = connection.send_raw("keyword input:kb_variant ").await?;
                ensure!(
                    result.trim() == "ok",
                    "Hyprland rejected the keyboard variant"
                );
            }
            let commands: Vec<_> = options(input)
                .into_iter()
                .filter(|(key, _)| *key != "kb_variant" || !input.variants.is_empty())
                .map(|(key, value)| format!("keyword input:{key} {}", value.render(false)))
                .collect();
            let result = connection
                .send_raw(&format!("[[BATCH]]{}", commands.join(";")))
                .await?;
            ensure!(
                result.split_whitespace().collect::<String>() == "ok".repeat(commands.len()),
                "Hyprland rejected one or more input options"
            );
        }
        Ok(())
    }

    pub(crate) async fn input_profile(
        &self,
        input: &Input,
        shortcuts: &Shortcuts,
    ) -> Result<(&'static str, String)> {
        input.validate()?;
        for chord in [
            &shortcuts.launcher,
            &shortcuts.windows,
            &shortcuts.tools,
            &shortcuts.lock,
        ] {
            crate::config::validate_input_shortcut(chord)?;
        }
        let connection = self.input_connection()?;
        let lua = uses_lua(connection).await?;
        let keys = bindings(input, shortcuts);
        if !keys.is_empty() {
            let current: Vec<ExistingBind> =
                serde_json::from_str(&connection.send_raw("j/binds").await?)?;
            check_conflicts(&current, &keys)?;
        }
        Ok((
            if lua {
                "hyprlume-input.lua"
            } else {
                "hyprlume-input.conf"
            },
            render(input, &keys, lua),
        ))
    }

    pub(crate) async fn next_layout(&self) -> Result<String> {
        let connection = self.input_connection()?;
        let reply = connection.send_raw("switchxkblayout current next").await?;
        ensure!(
            reply.trim() == "ok",
            "Hyprland rejected the keyboard layout switch"
        );
        self.input_devices()
            .await?
            .iter()
            .find(|keyboard| keyboard.main)
            .map(|keyboard| keyboard.active_keymap.clone())
            .filter(|name| !name.is_empty())
            .context("The active keyboard layout is unavailable")
    }
}
fn display_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_control())
        .take(160)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::display::tests::fixture;

    #[tokio::test]
    async fn legacy_apply_preserves_empty_variant_and_requires_every_acknowledgment() {
        let (_dir, desktop, server) = fixture(vec![
            ("j/status", r#"{"configProvider":"hyprlang"}"#.into()),
            ("keyword input:kb_variant ", "ok".into()),
            ("[[BATCH]]keyword input:kb_layout us;keyword input:repeat_rate 25;keyword input:repeat_delay 600;keyword input:sensitivity 0;keyword input:left_handed false;keyword input:natural_scroll false;keyword input:touchpad:natural_scroll false;keyword input:touchpad:tap-to-click true;keyword input:touchpad:disable_while_typing true", "ok".repeat(9)),
        ]).await;
        desktop.apply_input(&Input::default()).await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn profile_emits_allowlisted_lua_and_refuses_a_conflicting_compositor_bind() {
        let input = Input {
            system_key_bindings: true,
            ..Input::default()
        };
        for conflict in [false, true] {
            let binds = if conflict {
                r#"[{"key":"XF86AudioRaiseVolume","modmask":0,"dispatcher":"exec","arg":"other-control"}]"#
            } else {
                r#"[{"key":"XF86AudioRaiseVolume","modmask":0,"dispatcher":"lua","arg":"","description":"Hyprlume: hyprlume system-key volume-up"}]"#
            };
            let (_dir, desktop, server) = fixture(vec![
                ("j/status", r#"{"configProvider":"lua"}"#.into()),
                ("j/binds", binds.into()),
            ])
            .await;
            let result = desktop.input_profile(&input, &Shortcuts::default()).await;
            if conflict {
                assert!(result.unwrap_err().to_string().contains("already bound"));
            } else {
                let (name, profile) = result.unwrap();
                assert_eq!(name, "hyprlume-input.lua");
                assert!(profile.contains("hl.bind(\"XF86AudioRaiseVolume\", hl.dsp.exec_cmd(\"hyprlume system-key volume-up\")"));
                assert!(profile.contains("[\"input.touchpad.tap_to_click\"]=true"));
                assert!(!profile.contains("hyprlume launcher"));
            }
            server.await.unwrap();
        }
    }
}
