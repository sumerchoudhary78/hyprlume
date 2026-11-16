//! Deterministic English commands; unmatched text never becomes shell code.
use super::{Action, Plan};
use crate::config::Config;
use anyhow::{bail, Context, Result};

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
