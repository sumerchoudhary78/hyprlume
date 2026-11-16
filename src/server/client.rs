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
