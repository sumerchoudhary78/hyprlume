use super::*;
use std::os::unix::fs::PermissionsExt;
#[tokio::test]
async fn saves_are_validated_private_atomic_and_preserve_the_launch_allowlist() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let mut initial = Config::default();
    initial.apps.insert("terminal".into(), vec!["kitty".into()]);
    std::fs::write(&path, toml::to_string(&initial).unwrap()).unwrap();
    let controller = Controller::new(&initial, path.clone(), false);
    let mut values = controller.state().values;
    values.preferences.appearance.reduced_effects = true;
    let state = controller
        .handle(Request::Save {
            revision: 1,
            values: values.clone().into(),
        })
        .await
        .unwrap();
    assert!(!state.restart_required);
    assert_eq!(Config::load(Some(path.clone())).unwrap().apps, initial.apps);
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(controller
        .handle(Request::Save {
            revision: 1,
            values: values.clone().into()
        })
        .await
        .is_err());
    values.voice.threads = 4;
    let state = controller
        .handle(Request::Save {
            revision: state.revision,
            values: values.clone().into(),
        })
        .await
        .unwrap();
    assert!(state.restart_required);
    values.voice.threads = 99;
    let before = std::fs::read(&path).unwrap();
    assert!(controller
        .handle(Request::Save {
            revision: state.revision,
            values: values.into()
        })
        .await
        .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::write(&path, "# external edit\n").unwrap();
    assert!(controller
        .handle(Request::Save {
            revision: state.revision,
            values: state.values.into()
        })
        .await
        .unwrap_err()
        .to_string()
        .contains("outside"));
    let reloaded = controller.handle(Request::Reload).await.unwrap();
    assert!(reloaded.values.preferences.appearance.metrics);
    assert!(directory.path().read_dir().unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
}
#[test]
fn shortcuts_and_model_preferences_cannot_become_shell_or_config_syntax() {
    let mut config = Config::default();
    for key in [
        "SUPER+SPACE\nexec=bad",
        "SUPER+;sh",
        "SUPER+SUPER+L",
        "SPACE",
        "SUPER+F25",
    ] {
        config.preferences.shortcuts.launcher = key.into();
        assert!(validate_values(&config).is_err());
    }
    config.preferences = Preferences::default();
    config.llm.endpoint = Some("https://cloud.example/model".into());
    assert!(validate_values(&config).is_err());
    config.llm.endpoint = None;
    config.voice.asr_dir = "relative/model".into();
    assert!(validate_values(&config).is_err());
    let shortcuts = crate::config::Shortcuts::default();
    assert!(super::shortcuts::hyprlang(&shortcuts)
        .contains("bind = SUPER, SPACE, exec, hyprlume launcher\n"));
    assert!(super::shortcuts::lua(&shortcuts)
        .contains("hl.bind(\"SUPER + SPACE\", hl.dsp.exec_cmd(\"hyprlume launcher\"))"));
}
