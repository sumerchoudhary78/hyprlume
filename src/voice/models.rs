//! Model discovery and native recognizer configuration.
use crate::{config::Voice, protocol::VoiceMode};
use anyhow::{Context, Result};
use sherpa_onnx::{
    KeywordSpotter, KeywordSpotterConfig, OnlineModelConfig, OnlineRecognizer,
    OnlineRecognizerConfig,
};
use std::path::{Path, PathBuf};

pub(super) fn model(dir: &Path, prefix: &str) -> Result<PathBuf> {
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

pub(super) fn recognizer(config: &Voice) -> Result<OnlineRecognizer> {
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

pub(super) fn spotter(config: &Voice) -> Result<KeywordSpotter> {
    let config = KeywordSpotterConfig {
        model_config: online_model(&config.kws_dir, 1)?,
        keywords_file: Some(config.keywords_file.to_string_lossy().into_owned()),
        keywords_threshold: 0.35,
        ..Default::default()
    };
    KeywordSpotter::create(&config).context("could not load wake-word models")
}
