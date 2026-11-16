//! Loopback-only model transport with a bounded JSON action response.
use super::{Plan, MAX_ACTIONS};
use crate::config::Config;
use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use serde_json::json;

const MAX_RESPONSE_BYTES: usize = 32 * 1024;

pub(super) async fn plan(text: &str, config: &Config) -> Result<Plan> {
    let endpoint = config.llm.endpoint.as_deref().context("No direct command matched. Configure a local LLM, or try ‘workspace 2’, ‘focus firefox’ or ‘volume 30’.")?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(config.llm.timeout_seconds))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let request = json!({
        "model": config.llm.model,
        "temperature": 0,
        "max_tokens": 384,
        "messages": [
            {
                "role": "system",
                "content": format!(
                    "Return only an action plan matching the JSON schema. Never generate shell commands. Launchable app names: {:?}. Use only the user's explicit request.",
                    config.apps.keys().collect::<Vec<_>>()
                )
            },
            {"role": "user", "content": text}
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {"name": "desktop_plan", "strict": true, "schema": action_schema()}
        }
    });
    let response = client
        .post(endpoint)
        .json(&request)
        .send()
        .await?
        .error_for_status()?;
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES {
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

fn action_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["actions"],
        "properties": {
            "actions": {
                "type": "array",
                "minItems": 1,
                "maxItems": MAX_ACTIONS,
                "items": {
                    "oneOf": [
                        {
                            "type": "object", "additionalProperties": false,
                            "required": ["type", "id"],
                            "properties": {
                                "type": {"enum": ["workspace", "move_window"]},
                                "id": {"type": "integer", "minimum": 1, "maximum": 10}
                            }
                        },
                        {
                            "type": "object", "additionalProperties": false,
                            "required": ["type", "app"],
                            "properties": {
                                "type": {"enum": ["focus", "launch"]},
                                "app": {"type": "string", "maxLength": 64}
                            }
                        },
                        {
                            "type": "object", "additionalProperties": false,
                            "required": ["type", "percent"],
                            "properties": {
                                "type": {"const": "volume"},
                                "percent": {"type": "integer", "minimum": 0, "maximum": 100}
                            }
                        },
                        {
                            "type": "object", "additionalProperties": false,
                            "required": ["type", "muted"],
                            "properties": {
                                "type": {"const": "mute"},
                                "muted": {"type": "boolean"}
                            }
                        }
                    ]
                }
            }
        }
    })
}
