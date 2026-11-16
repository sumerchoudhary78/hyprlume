//! Bounded PipeWire capture and streaming wake/ASR recognition.
use super::{
    models::{recognizer, spotter},
    VoiceEvent,
};
use crate::{config::Voice, protocol::VoiceMode};
use anyhow::{Context, Result};
use sherpa_onnx::Wave;
use std::{
    path::Path,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{io::AsyncReadExt, process::Command, sync::mpsc};

const SAMPLE_RATE: i32 = 16_000;
// A 20 ms little-endian PCM frame, with eight frames of bounded backpressure.
const FRAME_SAMPLES: usize = 320;
const FRAME_BYTES: usize = FRAME_SAMPLES * 2;
type AudioFrame = [f32; FRAME_SAMPLES];

pub fn transcribe(config: &Voice, file: &Path) -> Result<String> {
    let wave = Wave::read(&file.to_string_lossy()).context("could not read PCM WAV")?;
    let recognizer = recognizer(config)?;
    let stream = recognizer.create_stream();
    stream.accept_waveform(wave.sample_rate(), wave.samples());
    stream.accept_waveform(
        wave.sample_rate(),
        &vec![0.0; wave.sample_rate() as usize / 2],
    );
    stream.input_finished();
    while recognizer.is_ready(&stream) {
        recognizer.decode(&stream);
    }
    Ok(recognizer
        .get_result(&stream)
        .context("ASR returned no result")?
        .text
        .trim()
        .to_owned())
}

pub fn detect_wake(config: &Voice, file: &Path) -> Result<String> {
    let wave = Wave::read(&file.to_string_lossy()).context("could not read PCM WAV")?;
    let spotter = spotter(config)?;
    let stream = spotter.create_stream();
    stream.accept_waveform(wave.sample_rate(), wave.samples());
    stream.accept_waveform(wave.sample_rate(), &vec![0.0; wave.sample_rate() as usize]);
    let mut keyword = String::new();
    while spotter.is_ready(&stream) {
        spotter.decode(&stream);
        if let Some(result) = spotter.get_result(&stream) {
            if !result.keyword.is_empty() {
                keyword = result.keyword;
            }
        }
    }
    Ok(keyword)
}

pub async fn listen(
    config: Voice,
    mode: VoiceMode,
    events: mpsc::Sender<VoiceEvent>,
    paused: Arc<AtomicBool>,
) -> Result<()> {
    let mut capture = Command::new("pw-record")
        .arg("--rate")
        .arg(SAMPLE_RATE.to_string())
        .args(["--channels", "1", "--format", "s16", "--raw", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("start PipeWire capture; install pipewire-audio")?;
    let mut audio = capture
        .stdout
        .take()
        .context("PipeWire audio unavailable")?;
    let (tx, rx) = mpsc::channel::<AudioFrame>(8);
    let mut worker = tokio::task::spawn_blocking(move || decode(config, mode, rx, events, paused));
    let mut bytes = [0u8; FRAME_BYTES];
    loop {
        tokio::select! {
            result = &mut worker => return result?,
            result = audio.read_exact(&mut bytes) => {
                result.context("PipeWire capture stopped")?;
                tx.send(pcm_frame(&bytes)).await.context("voice worker stopped")?;
            }
        }
    }
}

fn pcm_frame(bytes: &[u8; FRAME_BYTES]) -> AudioFrame {
    let mut frame = [0.0; FRAME_SAMPLES];
    for (sample, bytes) in frame.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *sample = i16::from_le_bytes(*bytes) as f32 / 32768.0;
    }
    frame
}

fn decode(
    config: Voice,
    mode: VoiceMode,
    mut audio: mpsc::Receiver<AudioFrame>,
    events: mpsc::Sender<VoiceEvent>,
    paused: Arc<AtomicBool>,
) -> Result<()> {
    let asr = recognizer(&config)?;
    let asr_stream = asr.create_stream();
    let kws = if mode == VoiceMode::Wake {
        Some(spotter(&config)?)
    } else {
        None
    };
    let keyword_stream = kws.as_ref().map(|s| s.create_stream());
    let mut awake = mode == VoiceMode::Dictate;
    let mut started = Instant::now();
    let mut last_partial = String::new();
    let mut partial_at = Instant::now();
    let mut was_paused = false;
    events.blocking_send(VoiceEvent::Ready)?;
    while let Some(frame) = audio.blocking_recv() {
        if paused.load(Ordering::Relaxed) {
            was_paused = true;
            continue;
        }
        if was_paused {
            asr.reset(&asr_stream);
            if let (Some(kws), Some(stream)) = (&kws, &keyword_stream) {
                kws.reset(stream);
            }
            awake = mode == VoiceMode::Dictate;
            was_paused = false;
        }
        if !awake {
            if let (Some(kws), Some(stream)) = (&kws, &keyword_stream) {
                stream.accept_waveform(SAMPLE_RATE, &frame);
                while kws.is_ready(stream) {
                    kws.decode(stream);
                    if kws
                        .get_result(stream)
                        .is_some_and(|r| !r.keyword.is_empty())
                    {
                        awake = true;
                        started = Instant::now();
                        asr.reset(&asr_stream);
                        kws.reset(stream);
                        events.blocking_send(VoiceEvent::Awake)?;
                        break;
                    }
                }
            }
            continue;
        }
        asr_stream.accept_waveform(SAMPLE_RATE, &frame);
        while asr.is_ready(&asr_stream) {
            asr.decode(&asr_stream);
        }
        let text = asr
            .get_result(&asr_stream)
            .map(|r| r.text.trim().chars().take(512).collect::<String>())
            .unwrap_or_default();
        if text != last_partial && partial_at.elapsed() >= Duration::from_millis(160) {
            events.blocking_send(VoiceEvent::Partial(text.clone()))?;
            last_partial = text.clone();
            partial_at = Instant::now();
        }
        if asr.is_endpoint(&asr_stream) || started.elapsed() > Duration::from_secs(15) {
            asr.reset(&asr_stream);
            last_partial.clear();
            started = Instant::now();
            awake = mode == VoiceMode::Dictate;
            if !text.is_empty() {
                events.blocking_send(VoiceEvent::Final(text))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{pcm_frame, FRAME_BYTES};

    #[test]
    fn pcm_frames_preserve_signed_little_endian_samples() {
        let mut bytes = [0; FRAME_BYTES];
        bytes[..6].copy_from_slice(&[0x00, 0x80, 0x00, 0x00, 0xff, 0x7f]);
        let frame = pcm_frame(&bytes);
        assert_eq!(&frame[..3], &[-1.0, 0.0, 32767.0 / 32768.0]);
        assert!(frame[3..].iter().all(|sample| *sample == 0.0));
    }
}
