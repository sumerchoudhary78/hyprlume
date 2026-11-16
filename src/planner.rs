use crate::config::Config;
use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Workspace { id: i32 },
    MoveWindow { id: i32 },
    Focus { app: String },
    Launch { app: String },
    Volume { percent: u8 },
    Mute { muted: bool },
}

impl Action {
    pub fn description(&self) -> String {
        match self {
            Self::Workspace { id } => format!("Switch to workspace {id}"),
            Self::MoveWindow { id } => format!("Move the active window to workspace {id}"),
            Self::Focus { app } => format!("Focus {app}"),
            Self::Launch { app } => format!("Launch {app}"),
            Self::Volume { percent } => format!("Set volume to {percent}%"),
            Self::Mute { muted } => if *muted { "Mute audio" } else { "Unmute audio" }.into(),
        }
    }

    pub fn validate(&self, config: &Config) -> Result<()> {
        match self {
            Self::Workspace { id } | Self::MoveWindow { id } if !(1..=10).contains(id) => {
                bail!("workspace must be between 1 and 10")
            }
            Self::Focus { app }
                if app.is_empty()
                    || app.len() > 64
                    || !app
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ' || c == '.') =>
            {
                bail!("invalid app name")
            }
            Self::Launch { app } if !config.apps.contains_key(app) => {
                bail!("app {app:?} is not in the configured launch allowlist")
            }
            Self::Volume { percent } if *percent > 100 => bail!("volume must be between 0 and 100"),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub actions: Vec<Action>,
    #[serde(default)]
    pub source: String,
}

impl Plan {
    pub fn validate(&self, config: &Config) -> Result<()> {
        if self.actions.is_empty() || self.actions.len() > 4 {
            bail!("a plan needs between one and four actions");
        }
        for action in &self.actions {
            action.validate(config)?;
        }
        Ok(())
    }
}

fn number(s: &str) -> Option<i32> {
    if let Ok(n) = s.parse() {
        return Some(n);
    }
    let words = s.replace('-', " ");
    fn small(s: &str) -> Option<i32> {
        Some(match s {
            "zero" => 0,
            "one" | "won" => 1,
            "two" | "too" | "to" => 2,
            "three" => 3,
            "four" | "for" => 4,
            "five" => 5,
            "six" => 6,
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            "ten" => 10,
            "eleven" => 11,
            "twelve" => 12,
            "thirteen" => 13,
            "fourteen" => 14,
            "fifteen" => 15,
            "sixteen" => 16,
            "seventeen" => 17,
            "eighteen" => 18,
            "nineteen" => 19,
            "twenty" => 20,
            "thirty" => 30,
            "forty" => 40,
            "fifty" => 50,
            "sixty" => 60,
            "seventy" => 70,
            "eighty" => 80,
            "ninety" => 90,
            "hundred" => 100,
            _ => return None,
        })
    }
    let words: Vec<_> = words.split_whitespace().collect();
    match words.as_slice() {
        [single] => small(single),
        ["one" | "a", "hundred"] => Some(100),
        [tens, units] => {
            let (tens, units) = (small(tens)?, small(units)?);
            if (20..=90).contains(&tens) && tens % 10 == 0 && (1..=9).contains(&units) {
                Some(tens + units)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn direct(text: &str, config: &Config) -> Result<Option<Plan>> {
    if text.len() > 2048 || text.chars().any(|c| c.is_control()) {
        bail!("command must be plain text under 2048 bytes");
    }
    let normalized = text
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .to_ascii_lowercase();
    let s = normalized.strip_prefix("please ").unwrap_or(&normalized);
    let workspace = [
        "switch to workspace ",
        "workspace ",
        "go to workspace ",
        "switch to work space ",
        "work space ",
        "go to work space ",
    ]
    .iter()
    .find_map(|prefix| s.strip_prefix(prefix));
    let moved = [
        "move window to workspace ",
        "move this window to workspace ",
        "move window to work space ",
        "move this window to work space ",
    ]
    .iter()
    .find_map(|prefix| s.strip_prefix(prefix));
    let action = if let Some(id) = workspace {
        Some(Action::Workspace {
            id: number(id).context("expected workspace number")?,
        })
    } else if let Some(id) = moved {
        Some(Action::MoveWindow {
            id: number(id).context("expected workspace number")?,
        })
    } else if let Some(app) = s.strip_prefix("focus ") {
        Some(Action::Focus { app: app.into() })
    } else if let Some(app) = s
        .strip_prefix("open ")
        .or_else(|| s.strip_prefix("launch "))
    {
        // A compound request belongs to the planner, not an executable string.
        if config.apps.contains_key(app) {
            Some(Action::Launch { app: app.into() })
        } else {
            None
        }
    } else if let Some(percent) = s
        .strip_prefix("set volume to ")
        .or_else(|| s.strip_prefix("set the volume to "))
        .or_else(|| s.strip_prefix("volume "))
    {
        let n: u8 = number(percent.trim_end_matches('%').trim_end_matches(" percent"))
            .and_then(|n| u8::try_from(n).ok())
            .context("expected volume from 0 to 100")?;
        Some(Action::Volume { percent: n })
    } else if matches!(s, "mute" | "mute audio") {
        Some(Action::Mute { muted: true })
    } else if matches!(s, "unmute" | "unmute audio") {
        Some(Action::Mute { muted: false })
    } else {
        None
    };
    action
        .map(|a| {
            a.validate(config)?;
            Ok(Plan {
                actions: vec![a],
                source: "direct".into(),
            })
        })
        .transpose()
}

pub async fn plan(text: &str, config: &Config) -> Result<Plan> {
    if let Some(plan) = direct(text, config)? {
        return Ok(plan);
    }
    let endpoint = config.llm.endpoint.as_deref().context("No direct command matched. Configure a local LLM, or try ‘workspace 2’, ‘focus firefox’ or ‘volume 30’.")?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(config.llm.timeout_seconds))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let schema = json!({"type":"object","additionalProperties":false,"required":["actions"],"properties":{"actions":{"type":"array","minItems":1,"maxItems":4,"items":{"oneOf":[
        {"type":"object","additionalProperties":false,"required":["type","id"],"properties":{"type":{"enum":["workspace","move_window"]},"id":{"type":"integer","minimum":1,"maximum":10}}},
        {"type":"object","additionalProperties":false,"required":["type","app"],"properties":{"type":{"enum":["focus","launch"]},"app":{"type":"string","maxLength":64}}},
        {"type":"object","additionalProperties":false,"required":["type","percent"],"properties":{"type":{"const":"volume"},"percent":{"type":"integer","minimum":0,"maximum":100}}},
        {"type":"object","additionalProperties":false,"required":["type","muted"],"properties":{"type":{"const":"mute"},"muted":{"type":"boolean"}}}
    ]}}}});
    let response = client.post(endpoint).json(&json!({
        "model":config.llm.model,"temperature":0,"max_tokens":384,
        "messages":[{"role":"system","content":format!("Return only an action plan matching the JSON schema. Never generate shell commands. Launchable app names: {:?}. Use only the user's explicit request.",config.apps.keys().collect::<Vec<_>>())},{"role":"user","content":text}],
        "response_format":{"type":"json_schema","json_schema":{"name":"desktop_plan","strict":true,"schema":schema}}
    })).send().await?.error_for_status()?;
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if bytes.len() + chunk.len() > 32768 {
            bail!("LLM response exceeded 32 KiB");
        }
        bytes.extend_from_slice(&chunk);
    }
    let response: serde_json::Value = serde_json::from_slice(&bytes)?;
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .context("LLM returned no JSON plan")?;
    let mut plan: Plan =
        serde_json::from_str(content).context("LLM returned an invalid action plan")?;
    plan.source = "local_llm".into();
    plan.validate(config)?;
    Ok(plan)
}
