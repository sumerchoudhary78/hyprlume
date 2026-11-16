//! CPU inference lives on a dedicated blocking worker. The PipeWire reader
//! feeds a bounded queue of 20 ms frames; no audio is saved to disk.
use crate::{config::Voice, protocol::VoiceMode};
use anyhow::{Context, Result};
use sherpa_onnx::{
    GenerationConfig, KeywordSpotter, KeywordSpotterConfig, OfflineTts, OfflineTtsConfig,
    OnlineModelConfig, OnlineRecognizer, OnlineRecognizerConfig, Wave,
};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{io::AsyncReadExt, process::Command, sync::mpsc};

pub enum VoiceEvent {
    Ready,
    Awake,
    Partial(String),
    Final(String),
}

fn model(dir: &Path, prefix: &str) -> Result<PathBuf> {
    let mut matches: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("models missing at {}; run hyprlume-models", dir.display()))?
        .filter_map(|v| v.ok())
        .map(|v| v.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(prefix))
                && p.extension().is_some_and(|n| n == "onnx")
        })
        .collect();
    matches.sort_by_key(|p| (!p.to_string_lossy().contains("int8"), p.clone()));
    matches
        .into_iter()
        .next()
        .with_context(|| format!("missing {prefix} ONNX model in {}", dir.display()))
}

fn online_model(dir: &Path, threads: i32) -> Result<OnlineModelConfig> {
    anyhow::ensure!(
        dir.join("tokens.txt").is_file(),
        "missing tokens.txt in {}",
        dir.display()
    );
    let mut config = OnlineModelConfig::default();
    config.transducer.encoder = Some(model(dir, "encoder")?.to_string_lossy().into_owned());
    config.transducer.decoder = Some(model(dir, "decoder")?.to_string_lossy().into_owned());
    config.transducer.joiner = Some(model(dir, "joiner")?.to_string_lossy().into_owned());
    config.tokens = Some(dir.join("tokens.txt").to_string_lossy().into_owned());
    config.num_threads = threads;
    Ok(config)
}

pub fn check_models(config: &Voice, mode: VoiceMode) -> Result<()> {
    online_model(&config.asr_dir, config.threads)?;
    if mode == VoiceMode::Wake {
        online_model(&config.kws_dir, 1)?;
        anyhow::ensure!(
            config.keywords_file.is_file(),
            "missing wake-word keywords file {}",
            config.keywords_file.display()
        );
    }
    Ok(())
}

fn recognizer(config: &Voice) -> Result<OnlineRecognizer> {
    let config = OnlineRecognizerConfig {
        model_config: online_model(&config.asr_dir, config.threads)?,
        enable_endpoint: true,
        rule1_min_trailing_silence: 2.4,
        rule2_min_trailing_silence: 0.6,
        rule3_min_utterance_length: 12.0,
        ..Default::default()
    };
    OnlineRecognizer::create(&config).context("could not load streaming English ASR models")
}

fn spotter(config: &Voice) -> Result<KeywordSpotter> {
    let config = KeywordSpotterConfig {
        model_config: online_model(&config.kws_dir, 1)?,
        keywords_file: Some(config.keywords_file.to_string_lossy().into_owned()),
        keywords_threshold: 0.35,
        ..Default::default()
    };
    KeywordSpotter::create(&config).context("could not load wake-word models")
}

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

pub struct Speaker {
    tts: OfflineTts,
}

impl Speaker {
    pub fn new(config: &Voice) -> Result<Self> {
        let mut tts = OfflineTtsConfig::default();
        tts.model.vits.model = Some(
            model(&config.tts_dir, "en_US-lessac-medium")?
                .to_string_lossy()
                .into_owned(),
        );
        let tokens = config.tts_dir.join("tokens.txt");
        anyhow::ensure!(tokens.is_file(), "missing TTS tokens.txt");
        tts.model.vits.tokens = Some(tokens.to_string_lossy().into_owned());
        tts.model.num_threads = config.threads;
        Ok(Self {
            tts: OfflineTts::create(&tts).context("could not load Piper TTS model")?,
        })
    }

    pub fn synthesize(&self, text: &str, output: &Path) -> Result<()> {
        anyhow::ensure!(
            !text.is_empty() && text.len() <= 2048 && !text.contains('\0'),
            "TTS text must contain 1–2048 bytes without NUL"
        );
        // eSpeak is a separate optional program, not a GPL-linked dependency.
        // It supplies IPA codepoints; Piper's neural model produces the audio.
        let mut child = std::process::Command::new("espeak-ng")
            .args(["-q", "--ipa=3", "-v", "en-us", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("install espeak-ng for TTS phonemization")?;
        child
            .stdin
            .take()
            .context("espeak stdin unavailable")?
            .write_all(text.as_bytes())?;
        let phonemes = child.wait_with_output()?;
        anyhow::ensure!(phonemes.status.success(), "espeak-ng failed");
        let phonemes = String::from_utf8(phonemes.stdout)?;
        let mut sentences: Vec<Vec<i32>> = phonemes
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                l.trim()
                    .chars()
                    .filter(|c| !matches!(c, '\u{200d}' | '\u{200c}'))
                    .map(|c| c as i32)
                    .collect()
            })
            .collect();
        for sentence in &mut sentences {
            sentence.push('.' as i32);
        }
        anyhow::ensure!(!sentences.is_empty(), "phonemizer returned no speech");
        let generation = GenerationConfig {
            silence_scale: 1.0,
            sid: 0,
            speed: 1.0,
            phoneme_codepoints: Some(sentences),
            ..Default::default()
        };
        let audio = self
            .tts
            .generate_with_config("", &generation, None::<fn(&[f32], f32) -> bool>)
            .context("Piper returned no speech")?;
        anyhow::ensure!(
            audio.save(&output.to_string_lossy()),
            "could not save {}",
            output.display()
        );
        Ok(())
    }
}

pub fn speak(config: &Voice, text: &str, output: &Path) -> Result<()> {
    Speaker::new(config)?.synthesize(text, output)
}

pub async fn listen(
    config: Voice,
    mode: VoiceMode,
    events: mpsc::Sender<VoiceEvent>,
    paused: Arc<AtomicBool>,
) -> Result<()> {
    let mut capture = Command::new("pw-record")
        .args([
            "--rate",
            "16000",
            "--channels",
            "1",
            "--format",
            "s16",
            "--raw",
            "-",
        ])
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
    let (tx, rx) = mpsc::channel::<[f32; 320]>(8);
    let mut worker = tokio::task::spawn_blocking(move || decode(config, mode, rx, events, paused));
    let mut bytes = [0u8; 640];
    loop {
        tokio::select! {
            result = &mut worker => return result?,
            result = audio.read_exact(&mut bytes) => {
                result.context("PipeWire capture stopped")?;
                let mut frame = [0.0; 320];
                for (sample, bytes) in frame.iter_mut().zip(bytes.as_chunks::<2>().0) { *sample = i16::from_le_bytes(*bytes) as f32 / 32768.0; }
                tx.send(frame).await.context("voice worker stopped")?;
            }
        }
    }
}

fn decode(
    config: Voice,
    mode: VoiceMode,
    mut audio: mpsc::Receiver<[f32; 320]>,
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
                stream.accept_waveform(16000, &frame);
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
        asr_stream.accept_waveform(16000, &frame);
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
