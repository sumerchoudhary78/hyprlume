use super::{flag, integer, text, Native, LOGIND};
use crate::services::power::{
    types::{Binding, SessionTarget},
    Session, SessionAction,
};
use anyhow::{ensure, Context, Result};
use zbus::zvariant::OwnedObjectPath;

async fn target(native: &Native, owner: &str, path: &str) -> Result<SessionTarget> {
    let binding = Binding {
        owner: owner.into(),
        path: path.into(),
        interface: "org.freedesktop.login1.Session".into(),
    };
    let props = native.properties(&binding).await?;
    let user: (u32, OwnedObjectPath) = props
        .get("User")
        .context("Session has no owner")?
        .try_clone()?
        .try_into()?;
    ensure!(
        user.0 == native.uid && !flag(&props, "Remote") && flag(&props, "Active"),
        "Session is remote, inactive or belongs to another user"
    );
    ensure!(
        matches!(text(&props, "Type", 16).as_str(), "wayland" | "x11")
            && matches!(text(&props, "Class", 16).as_str(), "user" | "user-early"),
        "Session is not a graphical user login"
    );
    ensure!(
        text(&props, "State", 16) != "closing",
        "Graphical session is closing"
    );
    let id = text(&props, "Id", 64);
    let timestamp = integer(&props, "TimestampMonotonic");
    ensure!(
        !id.is_empty() && timestamp > 0,
        "Graphical session identity is incomplete"
    );
    Ok(SessionTarget {
        binding,
        id,
        uid: user.0,
        timestamp,
    })
}
async fn selected(native: &Native, manager: &Binding) -> Result<SessionTarget> {
    let proxy = native.proxy(manager).await?;
    if let Some(id) = &native.session_hint {
        ensure!(
            id.len() <= 64
                && id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
            "Invalid graphical session id"
        );
        let path: OwnedObjectPath = proxy.call("GetSession", &(id,)).await?;
        return target(native, &manager.owner, path.as_str()).await;
    }
    let sessions: Vec<(String, u32, String, String, OwnedObjectPath)> =
        proxy.call("ListSessions", &()).await?;
    let mut selected = None;
    for (_, uid, _, seat, path) in sessions.iter().take(64) {
        if *uid != native.uid || seat.is_empty() {
            continue;
        }
        if let Ok(candidate) = target(native, &manager.owner, path.as_str()).await {
            ensure!(
                selected.is_none(),
                "Multiple graphical sessions; import XDG_SESSION_ID from the intended desktop"
            );
            selected = Some(candidate);
        }
    }
    selected.context("No active graphical login; import XDG_SESSION_ID from your desktop")
}
async fn snapshot(native: &Native) -> Result<Session> {
    let manager = native
        .binding(
            LOGIND,
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
        .await?;
    let target = selected(native, &manager).await?;
    let proxy = native.proxy(&manager).await?;
    async fn capability(proxy: &zbus::Proxy<'_>, method: &str) -> String {
        let result: zbus::Result<String> = proxy.call(method, &()).await;
        match result.as_deref() {
            Ok("yes") => "yes",
            Ok("challenge") => "challenge",
            Ok("inhibited" | "inhibitor-blocked" | "challenge-inhibitor-blocked") => "inhibited",
            _ => "no",
        }
        .into()
    }
    let (mut suspend, mut reboot, mut power_off) = tokio::join!(
        capability(&proxy, "CanSuspend"),
        capability(&proxy, "CanReboot"),
        capability(&proxy, "CanPowerOff")
    );
    let inhibitors: Vec<(String, String, String, String, u32, u32)> =
        proxy.call("ListInhibitors", &()).await?;
    for (what, _, _, mode, _, _) in &inhibitors {
        if !matches!(mode.as_str(), "block" | "block-weak") {
            continue;
        }
        if what.split(':').any(|item| item == "sleep") {
            suspend = "inhibited".into();
        }
        if what.split(':').any(|item| item == "shutdown") {
            reboot = "inhibited".into();
            power_off = "inhibited".into();
        }
    }
    Ok(Session {
        available: true,
        id: target.id.clone(),
        suspend,
        reboot,
        power_off,
        logout: "yes".into(),
        error: String::new(),
        target: Some(target),
    })
}
pub(super) async fn read(native: &Native) -> Session {
    snapshot(native).await.unwrap_or_else(|error| Session {
        error: error.to_string().chars().take(180).collect(),
        ..Session::default()
    })
}
pub(super) async fn verify(native: &Native, expected: &SessionTarget) -> Result<Session> {
    let current = snapshot(native).await?;
    ensure!(
        current.target.as_ref() == Some(expected),
        "Graphical session changed; review the action again"
    );
    Ok(current)
}
pub(super) async fn act(
    native: &Native,
    kind: SessionAction,
    target: SessionTarget,
) -> Result<String> {
    verify(native, &target).await?.validate(kind)?;
    if kind == SessionAction::Logout {
        native
            .proxy(&target.binding)
            .await?
            .call::<_, _, ()>("Terminate", &())
            .await?;
    } else {
        let manager = Binding {
            path: "/org/freedesktop/login1".into(),
            interface: "org.freedesktop.login1.Manager".into(),
            ..target.binding
        };
        let method = match kind {
            SessionAction::Suspend => "SuspendWithFlags",
            SessionAction::Reboot => "RebootWithFlags",
            SessionAction::PowerOff => "PowerOffWithFlags",
            SessionAction::Logout => unreachable!(),
        };
        // ROOT_CHECK_INHIBITORS honours block-weak locks too. Never set SKIP.
        native
            .proxy(&manager)
            .await?
            .call_with_flags::<_, _, ()>(
                method,
                zbus::proxy::MethodFlags::AllowInteractiveAuth.into(),
                &(1u64,),
            )
            .await?;
    }
    Ok(format!("{} request accepted by logind", kind.label()))
}
