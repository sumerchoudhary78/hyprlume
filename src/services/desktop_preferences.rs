//! Association changes delegate to GIO so the normal XDG precedence applies.
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Association { mime: String },
    SetAssociation { mime: String, desktop_id: String },
    WriteEnvironment { revision: u64 },
    StartScreenReader,
}

fn validate_type(mime: &str) -> Result<()> {
    let valid = |part: &str| {
        !part.is_empty()
            && part.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"!#$&^_.+-".contains(&byte)
            })
    };
    ensure!(
        mime.len() <= 128
            && mime
                .split_once('/')
                .is_some_and(|(group, subtype)| valid(group) && valid(subtype)),
        "Enter a MIME type such as application/pdf or x-scheme-handler/https"
    );
    Ok(())
}

fn validate_id(id: &str) -> Result<()> {
    ensure!(
        id.len() <= 180
            && id.ends_with(".desktop")
            && !id.starts_with(['.', '-'])
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)),
        "Choose an installed .desktop ID"
    );
    Ok(())
}

pub(crate) async fn association(
    mime: String,
    selected: Option<String>,
) -> Result<serde_json::Value> {
    validate_type(&mime)?;
    if let Some(id) = &selected {
        validate_id(id)?;
    }
    #[cfg(target_os = "linux")]
    {
        tokio::task::spawn_blocking(move || native::association(&mime, selected.as_deref())).await?
    }
    #[cfg(not(target_os = "linux"))]
    anyhow::bail!("Default app controls require Linux and GIO")
}

pub(crate) async fn start_reader() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        tokio::task::spawn_blocking(native::start_reader).await?
    }
    #[cfg(not(target_os = "linux"))]
    anyhow::bail!("Orca requires a Linux graphical session")
}

#[cfg(target_os = "linux")]
mod native {
    use anyhow::{ensure, Context, Result};
    use gio::{prelude::AppInfoExt, AppInfo, AppLaunchContext, DesktopAppInfo, File};
    use serde_json::{json, Value};

    pub(super) fn association(mime: &str, selected: Option<&str>) -> Result<Value> {
        let scheme = mime.starts_with("x-scheme-handler/");
        if let Some(id) = selected {
            let app =
                DesktopAppInfo::new(id).context("This desktop application is not installed")?;
            ensure!(
                !app.is_hidden() && (!scheme || app.supports_uris()),
                "This application cannot handle the requested type"
            );
            app.set_as_default_for_type(mime)
                .context("Write the default application through GIO")?;
        }
        let default = AppInfo::default_for_type(mime, false);
        let default_id = default.as_ref().and_then(AppInfoExt::id);
        if let Some(id) = selected {
            ensure!(
                default_id.as_deref() == Some(id),
                "A higher-priority association still controls this type"
            );
        }
        let apps: Vec<_> = AppInfo::all_for_type(mime).into_iter()
            .filter(AppInfoExt::should_show)
            .filter_map(|app| {
                let id = app.id()?;
                super::validate_id(&id).ok()?;
                Some(json!({"id":id.as_str(), "name":app.display_name().chars().filter(|c| !c.is_control()).take(100).collect::<String>()}))
            }).take(16).collect();
        Ok(json!({
            "mime":mime, "desktop_id":default_id.as_deref(),
            "app_name":default.as_ref().map(|app| app.display_name().chars().filter(|c| !c.is_control()).take(100).collect::<String>()),
            "choices":apps,
            "message": if selected.is_some() { "Default application saved." } else { "Current XDG application association." },
        }))
    }

    pub(super) fn start_reader() -> Result<()> {
        let orca = DesktopAppInfo::new("org.gnome.Orca.desktop")
            .or_else(|| DesktopAppInfo::new("orca.desktop"))
            .context("Install Orca to start the screen reader")?;
        orca.launch(&[] as &[File], None::<&AppLaunchContext>)
            .context("Could not start Orca")?;
        Ok(())
    }
}
