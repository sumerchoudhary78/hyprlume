//! Voice lifecycle, UI events and feedback coordination. No native model internals.
use super::App;
use crate::{config::socket_path, protocol::VoiceMode, voice::VoiceEvent};
use anyhow::{bail, Context, Result};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::timeout;

impl App {
    pub(super) async fn feedback(&self) -> Result<()> {
        struct Pause(Arc<AtomicBool>);
        impl Drop for Pause {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Relaxed);
            }
        }
        let epoch = self.locking.epoch();
        self.check_epoch(epoch)?;
        let mut locking = self.locking.subscribe();
        self.audio_paused.store(true, Ordering::Relaxed);
        let _pause = Pause(self.audio_paused.clone());
        let output = socket_path()?.parent().unwrap().join("feedback.wav");
        let cache = self.speaker.clone();
        let config = self.config.voice.clone();
        let text: String = self.state().receipts.join(". ").chars().take(400).collect();
        let path = output.clone();
        self.update(|s| s.phase = "speaking".into());
        tokio::task::spawn_blocking(move || {
            let mut speaker = cache
                .lock()
                .map_err(|_| anyhow::anyhow!("TTS cache lock failed"))?;
            if speaker.is_none() {
                *speaker = Some(crate::voice::Speaker::new(&config)?);
            }
            speaker.as_ref().unwrap().synthesize(&text, &path)
        })
        .await??;
        if let Err(error) = self.check_epoch(epoch) {
            let _ = fs::remove_file(&output);
            return Err(error);
        }
        let playback = tokio::process::Command::new("pw-play")
            .arg(&output)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .status();
        let played = tokio::select! {
            result=timeout(Duration::from_secs(20),playback)=>result,
            _=async {while !locking.borrow().active {if locking.changed().await.is_err(){break;}}}=>{
                let _=fs::remove_file(&output);
                bail!("Speech playback stopped for session locking");
            }
        };
        let _ = fs::remove_file(output);
        if !played.context("speech playback timed out")??.success() {
            bail!("PipeWire playback failed");
        }
        // Drain microphone echo/backlog before arming the keyword spotter again.
        tokio::time::sleep(Duration::from_millis(300)).await;
        self.update(|s| s.phase = "done".into());
        Ok(())
    }

    pub(super) async fn set_voice(self: &Arc<Self>, mode: VoiceMode) -> Result<()> {
        let epoch = self.locking.epoch();
        if mode != VoiceMode::Off {
            self.check_epoch(epoch)?;
        }
        let mut voice = self.voice.lock().await;
        if mode != VoiceMode::Off {
            self.check_epoch(epoch)?;
        }
        if let Some(task) = voice.take() {
            task.abort();
            let _ = task.await;
        }
        self.update(|s| {
            s.voice = VoiceMode::Off;
            if s.confirmation.is_none()
                && !matches!(s.phase.as_str(), "planning" | "executing" | "speaking")
            {
                s.phase = "ready".into();
                s.message = "Microphone off".into();
            }
        });
        if mode == VoiceMode::Off {
            return Ok(());
        }
        self.check_epoch(epoch)?;
        if self.desktop.demo {
            bail!("microphone capture is unavailable in demo mode");
        }
        crate::voice::check_models(&self.config.voice, mode)?;
        self.check_epoch(epoch)?;
        self.update(|s| {
            s.voice = mode;
            if s.confirmation.is_none()
                && !matches!(s.phase.as_str(), "planning" | "executing" | "speaking")
            {
                s.message = "Starting local microphone capture".into();
            }
        });
        let app = self.clone();
        *voice = Some(tokio::spawn(async move { app.run_voice(mode).await }));
        Ok(())
    }

    async fn run_voice(self: Arc<Self>, mode: VoiceMode) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let worker = crate::voice::listen(
            self.config.voice.clone(),
            mode,
            tx,
            self.audio_paused.clone(),
        );
        tokio::pin!(worker);
        let result = loop {
            tokio::select! {
                result = &mut worker => break result,
                event = rx.recv() => {
                    let Some(event) = event else { break Ok(()); };
                    if let Err(error) = self.handle_voice_event(mode, event).await {
                        self.fail(&error.to_string());
                    }
                }
            }
        };
        if let Err(error) = result {
            let message = format!("Voice stopped: {error}");
            if self.accepts_voice_input() {
                self.fail(&message);
            } else {
                self.update(|state| state.message = message.chars().take(512).collect());
            }
        }
        self.update(|state| state.voice = VoiceMode::Off);
    }

    fn accepts_voice_input(&self) -> bool {
        !self.locking.active()
            && self.busy.available_permits() > 0
            && self.state().confirmation.is_none()
    }

    async fn handle_voice_event(
        self: &Arc<Self>,
        mode: VoiceMode,
        event: VoiceEvent,
    ) -> Result<()> {
        // A second wake cannot replace a preview or running command.
        if !self.accepts_voice_input() {
            return Ok(());
        }
        match event {
            VoiceEvent::Ready => self.update(|state| {
                state.voice = mode;
                state.phase = "ready".into();
                state.message = if mode == VoiceMode::Wake {
                    "Listening for ‘Hey Pilot’"
                } else {
                    "Dictation · text stays in this panel"
                }
                .into();
            }),
            VoiceEvent::Awake => self.update(|state| {
                state.phase = "listening".into();
                state.panel_open = true;
                state.transcript.clear();
                state.message = "Listening to your command".into();
            }),
            VoiceEvent::Partial(text) => self.update(|state| state.transcript = text),
            VoiceEvent::Final(text) => {
                if mode == VoiceMode::Dictate {
                    // Dictation is text only; it never enters the action pipeline.
                    self.update(|state| {
                        state.phase = "ready".into();
                        state.transcript = text;
                        state.message = "Dictation ready · no desktop action".into();
                        state.panel_open = true;
                    });
                } else {
                    self.submit(text, !self.config.voice.execute_commands)
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{App, VoiceEvent, VoiceMode};
    use crate::{config::Config, protocol::Operation};

    #[tokio::test]
    async fn dictation_final_is_text_even_when_it_looks_like_a_command() {
        let app = App::new(Config::default(), true);
        app.handle_voice_event(VoiceMode::Dictate, VoiceEvent::Final("workspace 2".into()))
            .await
            .unwrap();
        let state = app.state();
        assert_eq!(state.transcript, "workspace 2");
        assert_eq!(state.phase, "ready");
        assert!(state.panel_open);
        assert!(state.confirmation.is_none());
        assert!(state.actions.is_empty() && state.receipts.is_empty());
    }

    #[tokio::test]
    async fn voice_events_preserve_a_pending_preview() {
        let app = App::new(Config::default(), true);
        app.handle(Operation::Command {
            text: "workspace 2".into(),
            dry_run: true,
        })
        .await
        .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while app.state().phase != "review" {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let before = serde_json::to_value(app.state()).unwrap();
        for event in [
            VoiceEvent::Ready,
            VoiceEvent::Awake,
            VoiceEvent::Partial("volume 90".into()),
            VoiceEvent::Final("volume 90".into()),
        ] {
            app.handle_voice_event(VoiceMode::Wake, event)
                .await
                .unwrap();
            assert_eq!(serde_json::to_value(app.state()).unwrap(), before);
        }
    }

    #[tokio::test]
    async fn voice_events_preserve_an_in_flight_command() {
        let app = App::new(Config::default(), true);
        let _permit = app.busy.clone().try_acquire_owned().unwrap();
        app.update(|state| {
            state.phase = "executing".into();
            state.transcript = "workspace 2".into();
        });
        let before = serde_json::to_value(app.state()).unwrap();
        for event in [
            VoiceEvent::Ready,
            VoiceEvent::Awake,
            VoiceEvent::Partial("other".into()),
            VoiceEvent::Final("mute".into()),
        ] {
            app.handle_voice_event(VoiceMode::Wake, event)
                .await
                .unwrap();
            assert_eq!(serde_json::to_value(app.state()).unwrap(), before);
        }
    }
}
