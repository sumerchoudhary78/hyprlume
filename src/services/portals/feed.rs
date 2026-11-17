//! Follow saved appearance settings over the daemon's existing read-only topic.
use crate::{
    config::{Accent, AppColor},
    protocol::{Operation, Request},
};
use anyhow::{ensure, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::{path::Path, time::Duration};
use tokio::{
    net::UnixStream,
    sync::watch,
    time::{sleep, timeout},
};
use tokio_util::codec::{Framed, LinesCodec};

type Stream = Framed<UnixStream, LinesCodec>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub(super) struct Palette {
    pub accent: Accent,
    pub reduced_effects: bool,
    #[serde(default)]
    pub high_contrast: bool,
    #[serde(default)]
    pub app_color: AppColor,
}

#[derive(Deserialize)]
struct Settings {
    values: Values,
}
#[derive(Deserialize)]
struct Values {
    preferences: Preferences,
}
#[derive(Deserialize)]
struct Preferences {
    appearance: Palette,
}
#[derive(Deserialize)]
struct Packet {
    id: Option<u64>,
    event: Option<String>,
    result: Option<Settings>,
    state: Option<Settings>,
}

async fn receive(stream: &mut Stream) -> Result<Packet> {
    let line = stream
        .next()
        .await
        .context("Settings feed disconnected")??;
    Ok(serde_json::from_str(&line)?)
}

pub(super) async fn connect(path: &Path) -> Result<(Stream, Palette)> {
    timeout(Duration::from_millis(750), async {
        let mut stream = Framed::new(
            UnixStream::connect(path).await?,
            LinesCodec::new_with_max_length(64 * 1024),
        );
        stream
            .send(serde_json::to_string(&Request {
                id: 1,
                operation: Operation::SettingsSubscribe,
            })?)
            .await?;
        let packet = receive(&mut stream).await?;
        ensure!(packet.id == Some(1), "Unexpected Settings reply");
        let settings = packet.result.context("Missing Settings snapshot")?;
        Ok((stream, settings.values.preferences.appearance))
    })
    .await
    .context("Settings feed did not answer")?
}

pub(super) async fn follow(
    path: &Path,
    mut stream: Option<Stream>,
    updates: watch::Sender<Palette>,
) {
    let update = |next| {
        updates.send_if_modified(|current| {
            if *current == next {
                return false;
            }
            *current = next;
            true
        });
    };
    let mut retry = Duration::from_secs(1);
    loop {
        if let Some(mut active) = stream.take() {
            while let Ok(packet) = receive(&mut active).await {
                if packet.event.as_deref() == Some("settings") {
                    if let Some(settings) = packet.state {
                        update(settings.values.preferences.appearance);
                    }
                }
            }
        }
        sleep(retry).await;
        match connect(path).await {
            Ok((active, palette)) => {
                update(palette);
                stream = Some(active);
                retry = Duration::from_secs(1);
            }
            Err(_) => retry = (retry * 2).min(Duration::from_secs(30)),
        }
    }
}
