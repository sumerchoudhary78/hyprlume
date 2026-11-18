mod desktop;
pub use desktop::{AppColor, DesktopPreferences, QtStyle};
mod input;
mod preferences;
pub use input::Input;
pub(crate) use preferences::validate_shortcut as validate_input_shortcut;
pub use preferences::{Accent, Appearance, Idle, Preferences, Services, Shortcuts};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// A name maps to argv, never to a shell command.
    pub apps: BTreeMap<String, Vec<String>>,
    pub llm: Llm,
    pub voice: Voice,
    pub preferences: Preferences,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Llm {
    pub endpoint: Option<String>,
    pub model: String,
    pub timeout_seconds: u64,
}

impl Default for Llm {
    fn default() -> Self {
        Self {
            endpoint: None,
            model: "local".into(),
            timeout_seconds: 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Voice {
    pub asr_dir: PathBuf,
    pub kws_dir: PathBuf,
    pub tts_dir: PathBuf,
    pub keywords_file: PathBuf,
    /// Voice commands preview by default. Explicitly enable reversible actions.
    pub execute_commands: bool,
    pub speak_results: bool,
    pub threads: i32,
}

impl Default for Voice {
    fn default() -> Self {
        let base = data_dir().join("models");
        Self {
            asr_dir: base.join("asr"),
            kws_dir: base.join("kws"),
            tts_dir: base.join("tts"),
            keywords_file: base.join("kws/hyprlume-keywords.txt"),
            execute_commands: false,
            speak_results: true,
            threads: 2,
        }
    }
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
        .join("hyprlume")
}

pub(crate) fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
}

pub fn config_path() -> PathBuf {
    config_dir().join("hyprlume/config.toml")
}

pub fn socket_path() -> Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR is required; run inside your user session")?;
    Ok(PathBuf::from(runtime).join("hyprlume/control.sock"))
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(config_path);
        if !path.exists() {
            return Ok(Self::default());
        }
        let config: Self = toml::from_str(
            &std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
        )?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        self.preferences.validate()?;
        if !(1..=4).contains(&self.voice.threads) {
            bail!("voice.threads must be between 1 and 4");
        }
        if !(1..=120).contains(&self.llm.timeout_seconds) {
            bail!("llm.timeout_seconds must be between 1 and 120");
        }
        for (name, argv) in &self.apps {
            if name.is_empty()
                || name.len() > 64
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ')
            {
                bail!("app names must contain only ASCII letters, digits, spaces or dashes");
            }
            if argv.is_empty() || argv[0].is_empty() || argv.iter().any(|s| s.contains('\0')) {
                bail!("app {name} needs nonempty argv");
            }
            if name != &name.to_ascii_lowercase() {
                bail!("app name {name} must be lowercase");
            }
        }
        if let Some(endpoint) = &self.llm.endpoint {
            let url = reqwest::Url::parse(endpoint)?;
            if url.scheme() != "http"
                || !matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "::1"))
                || !url.username().is_empty()
                || url.password().is_some()
            {
                bail!("v0.1 accepts only a local HTTP LLM endpoint at 127.0.0.1 or ::1");
            }
        }
        Ok(())
    }
}
