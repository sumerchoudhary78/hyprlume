//! Session credential lifecycle. Only collection metadata enters the shell.
#[cfg(target_os = "linux")]
mod native;

use crate::services::locking::Controller;
use anyhow::Result;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "linux")]
use std::time::Duration;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Collection {
    Login,
    Default,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Refresh,
    Activate,
    Lock { collection: Collection },
    Unlock { collection: Collection },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    #[default]
    Unknown,
    Missing,
    OnDemand,
    Running,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct State {
    pub provider: Provider,
    pub login_locked: Option<bool>,
    pub default_locked: Option<bool>,
    pub busy: bool,
    pub message: String,
    pub error: String,
}

pub(crate) async fn handle(locking: &Controller, epoch: u64, request: Request) -> Result<State> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (locking, epoch, request);
        anyhow::bail!("Secret Service integration requires a Linux desktop session")
    }
    #[cfg(target_os = "linux")]
    {
        let deadline = if matches!(request, Request::Lock { .. } | Request::Unlock { .. }) {
            Duration::from_secs(130)
        } else {
            Duration::from_secs(5)
        };
        tokio::time::timeout(deadline, native::handle(locking, epoch, request))
            .await
            .map_err(|_| anyhow::anyhow!("The credential provider did not respond in time"))?
    }
}
