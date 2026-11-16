//! Literal argv launches in independent systemd user units.
use super::run;
use anyhow::{Context, Result};

pub(super) async fn launch(app: &str, argv: &[String]) -> Result<String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let unit = format!(
        "hyprlume-app-{}-{}-{stamp}",
        app.replace(' ', "-"),
        std::process::id()
    );
    // The user manager owns the app independently of this daemon.
    // A daemon restart must not terminate an app or inherit its
    // service sandbox. Type=exec confirms that exec itself worked.
    let mut args = vec![
        "--user".to_owned(),
        "--collect".into(),
        "--quiet".into(),
        "--service-type=exec".into(),
        "--expand-environment=no".into(),
        format!("--unit={unit}"),
    ];
    for name in [
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "XDG_RUNTIME_DIR",
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_TYPE",
        "DBUS_SESSION_BUS_ADDRESS",
        "XAUTHORITY",
        "LANG",
        "LC_ALL",
        "PATH",
        "QT_QPA_PLATFORM",
        "QT_QPA_PLATFORMTHEME",
        "GDK_BACKEND",
        "SSH_AUTH_SOCK",
    ] {
        if std::env::var_os(name).is_some() {
            args.push(format!("--setenv={name}"));
        }
    }
    args.push("--".into());
    args.extend(argv.iter().cloned());
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    run("systemd-run", &args)
        .await
        .with_context(|| format!("launch {app} through the systemd user manager"))?;
    Ok(format!("{app} launch requested · process started"))
}
