//! One-shot CLI requests over the same bounded wire protocol as the shell.
use crate::protocol::{Operation, Request};
use anyhow::{bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::{path::Path, time::Duration};
use tokio::{net::UnixStream, time::timeout};
use tokio_util::codec::{Framed, LinesCodec};

pub async fn call(path: &Path, operation: Operation) -> Result<Value> {
    let stream = timeout(Duration::from_secs(2), UnixStream::connect(path))
        .await?
        .with_context(|| format!("connect {}; start ‘hyprlume daemon’ first", path.display()))?;
    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(64 * 1024));
    framed
        .send(serde_json::to_string(&Request { id: 1, operation })?)
        .await?;
    let response = timeout(Duration::from_secs(3), framed.next())
        .await?
        .context("daemon closed connection")??;
    let response: Value = serde_json::from_str(&response)?;
    if let Some(error) = response.get("error") {
        bail!("{}", error.as_str().unwrap_or("request failed"));
    }
    Ok(response["result"].clone())
}

/// Wait on pushed lock state; startup acknowledgment alone is not a lock.
pub async fn wait_for_lock(path: &Path) -> Result<()> {
    timeout(Duration::from_secs(45), async {
        let stream = UnixStream::connect(path).await?;
        let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(64 * 1024));
        framed
            .send(serde_json::to_string(&Request {
                id: 1,
                operation: Operation::LockSubscribe,
            })?)
            .await?;
        while let Some(message) = framed.next().await {
            let message: Value = serde_json::from_str(&message?)?;
            if let Some(error) = message.get("error") {
                bail!("{}", error.as_str().unwrap_or("Lock status failed"));
            }
            let value = message
                .get("result")
                .or_else(|| message.get("state"))
                .context("Invalid lock response")?;
            let state: crate::services::locking::State = serde_json::from_value(value.clone())?;
            if state.phase == "locked" && state.active {
                return Ok(());
            }
            if matches!(state.phase.as_str(), "error" | "recovery" | "idle") {
                bail!("{}", state.message);
            }
        }
        bail!("Daemon disconnected before confirming the lock")
    })
    .await
    .context("Timed out waiting for compositor lock confirmation")?
}
