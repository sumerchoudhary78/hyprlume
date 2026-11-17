//! Bounded Unix transport. Command and voice state belong to `app`.
mod client;
mod socket;

pub use crate::config::socket_path;
pub use client::{call, wait_for_lock};
// Keep the existing library entry point while `app::App` owns orchestration.
pub use crate::app::App;

use crate::{
    config::Config,
    protocol::{Operation, Request, VoiceMode},
};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{net::UnixStream, sync::Semaphore, time::timeout};
use tokio_util::codec::{Framed, LinesCodec};

pub const MAX_FRAME: usize = 8192;

async fn session(stream: UnixStream, app: Arc<App>) -> Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(MAX_FRAME));
    let mut state = app.subscribe();
    let mut connectivity = app.connectivity.subscribe();
    let mut notifications = app.notifications.subscribe();
    let mut locking = app.locking.subscribe();
    let mut lock_subscribed = false;
    let mut settings = app.settings.subscribe();
    let mut settings_subscribed = false;
    let mut clipboard = app.clipboard.subscribe();
    let mut capture = app.capture.subscribe();
    let mut clipboard_subscribed = false;
    let mut capture_subscribed = false;
    let mut power = app.power.subscribe();
    let mut subscribed = false;
    let mut connectivity_subscribed = false;
    let mut notifications_subscribed = false;
    let mut power_view = None;
    loop {
        let output = tokio::select! {
            input = framed.next() => {
                let Some(input) = input else { return Ok(()); };
                let input = input?;
                match serde_json::from_str::<Request>(&input) {
                    Ok(request) => {
                        if matches!(request.operation, Operation::Subscribe) { subscribed = true; state.borrow_and_update(); }
                        if matches!(request.operation, Operation::ConnectivitySubscribe) { connectivity_subscribed = true; connectivity.borrow_and_update(); }
                        if matches!(request.operation, Operation::NotificationsSubscribe) { notifications_subscribed = true; notifications.borrow_and_update(); }
                        if matches!(request.operation,Operation::LockSubscribe) {lock_subscribed=true; locking.borrow_and_update();}
                        if matches!(request.operation,Operation::SettingsSubscribe) {settings_subscribed=true; settings.borrow_and_update();}
                        if matches!(request.operation,Operation::ClipboardSubscribe) {clipboard_subscribed=true; clipboard.borrow_and_update();}
                        if matches!(request.operation,Operation::CaptureSubscribe) {capture_subscribed=true; capture.borrow_and_update();}
                        if matches!(request.operation, Operation::PowerSubscribe) {
                            if power_view.is_none() { power_view = Some(app.power.observe_ui()); }
                            power.borrow_and_update();
                        }
                        match app.handle(request.operation).await {
                            Ok(result) => json!({"id":request.id,"result":result}),
                            Err(e) => json!({"id":request.id,"error":e.to_string()}),
                        }
                    }
                    Err(_) => json!({"id":null,"error":"invalid request"}),
                }
            }
            change = state.changed(), if subscribed => {
                change?;
                let snapshot = state.borrow_and_update().clone();
                json!({"event":"state","state":snapshot})
            }
            change = connectivity.changed(), if connectivity_subscribed => {
                change?;
                let snapshot = connectivity.borrow_and_update().clone();
                json!({"event":"connectivity","state":snapshot})
            }
            change = notifications.changed(), if notifications_subscribed => {
                change?;
                let snapshot = notifications.borrow_and_update().clone();
                json!({"event":"notifications","state":snapshot})
            }
            change=locking.changed(), if lock_subscribed => {
                change?; let snapshot=locking.borrow_and_update().clone(); json!({"event":"locking","state":snapshot})
            }
            change=settings.changed(), if settings_subscribed => {
                change?; let snapshot=settings.borrow_and_update().clone(); json!({"event":"settings","state":snapshot})
            }
            change=clipboard.changed(), if clipboard_subscribed => {
                change?; let snapshot=clipboard.borrow_and_update().clone(); json!({"event":"clipboard","state":snapshot})
            }
            change=capture.changed(), if capture_subscribed => {
                change?; let snapshot=capture.borrow_and_update().clone(); json!({"event":"capture","state":snapshot})
            }
            change = power.changed(), if power_view.is_some() => {
                change?;
                let snapshot = power.borrow_and_update().clone();
                json!({"event":"power","state":snapshot})
            }
        };
        timeout(Duration::from_secs(2), framed.send(output.to_string())).await??;
    }
}

pub async fn serve(path: PathBuf, config: Config, demo: bool) -> Result<()> {
    serve_with_config(path, config, demo, crate::config::config_path()).await
}
pub async fn serve_with_config(
    path: PathBuf,
    config: Config,
    demo: bool,
    config_path: PathBuf,
) -> Result<()> {
    config.validate()?;
    let (listener, _guard) = socket::bind(&path)?;
    let app = App::with_paths(config, demo, path.clone(), config_path);
    app.notifications
        .handle(crate::services::notifications::Request::Dnd {
            enabled: app.config.preferences.services.notifications_dnd,
        })
        .await?;
    if !app.locking.active() && app.config.preferences.services.clipboard_history {
        if let Err(error) = app
            .clipboard
            .handle(crate::services::clipboard::Request::Enable { enabled: true })
            .await
        {
            tracing::warn!(%error,"Clipboard startup preference could not be applied");
        }
    }
    let mut services = tokio::task::JoinSet::new();
    services.spawn(app.clone().watch_desktop());
    services.spawn(app.connectivity.clone().run());
    services.spawn(app.notifications.clone().run());
    services.spawn(app.power.clone().run());
    services.spawn(app.locking.clone().run());
    services.spawn(app.clone().watch_lock());
    let connections = Arc::new(Semaphore::new(16));
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tracing::info!(socket = %path.display(), "hyprlume ready; microphone off");
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = terminate.recv() => break,
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let Ok(permit) = connections.clone().try_acquire_owned() else { continue; };
                let app = app.clone();
                tokio::spawn(async move { if let Err(e) = session(stream, app).await { tracing::debug!(error = %e, "client disconnected"); } drop(permit); });
            }
        }
    }
    app.handle(Operation::Voice {
        mode: VoiceMode::Off,
    })
    .await
    .ok();
    app.clipboard.shutdown().await;
    app.capture.shutdown().await;
    services.abort_all();
    Ok(())
}
