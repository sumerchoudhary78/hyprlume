#![cfg(feature = "voice")]

use hyprlume::{
    config::Config,
    planner::{direct, Action},
    voice,
};

#[test]
#[ignore = "requires the opt-in English model bundle and espeak-ng"]
fn english_speech_round_trip_and_wake_word() {
    let config = Config::default();
    let dir = tempfile::tempdir().unwrap();
    let command = dir.path().join("command.wav");
    let wake = dir.path().join("wake.wav");
    voice::speak(&config.voice, "Go to work space two.", &command).unwrap();
    assert!(std::fs::metadata(&command).unwrap().len() > 1000);
    let transcript = voice::transcribe(&config.voice, &command).unwrap();
    eprintln!("TTS → ASR: {transcript}");
    assert_eq!(
        direct(&transcript, &config).unwrap().unwrap().actions,
        [Action::Workspace { id: 2 }],
        "{transcript}"
    );
    voice::speak(&config.voice, "Hey Pilot.", &wake).unwrap();
    let keyword = voice::detect_wake(&config.voice, &wake).unwrap();
    assert!(!keyword.is_empty(), "wake keyword was not detected");
    // A desktop command on its own must not open the wake gate.
    assert!(voice::detect_wake(&config.voice, &command)
        .unwrap()
        .is_empty());
}
