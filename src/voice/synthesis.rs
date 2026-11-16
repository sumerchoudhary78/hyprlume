//! Piper synthesis with eSpeak phonemization; playback belongs to the app.
use super::models::model;
use crate::config::Voice;
use anyhow::{Context, Result};
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig};
use std::{io::Write, path::Path, process::Stdio};

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
