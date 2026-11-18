//! Preview transaction supervisor. Closing stdin reverts the tested layout.
use super::{
    layout::{self, Output, Plan, Setting},
    profiles, Preview, PREVIEW_MS,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    time::Duration,
};

#[derive(Deserialize, Serialize)]
pub(crate) struct Job {
    pub plan: Plan,
    pub profile: PathBuf,
    pub token: u64,
    pub automatic: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Reply {
    Applied { preview: Preview },
    Done { message: String, error: String },
}

pub(super) async fn outputs() -> Result<Vec<Output>> {
    ensure!(
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
        "Display settings need a Wayland session"
    );
    let text = crate::desktop::run("wlr-randr", &["--json"])
        .await
        .context("Read outputs with wlr-randr")?;
    layout::parse(&text)
}

async fn apply(settings: &[Setting]) -> Result<()> {
    if settings.is_empty() {
        return Ok(());
    }
    let mut args = Vec::new();
    for setting in settings {
        args.extend([
            "--output".into(),
            setting.name.clone(),
            "--mode".into(),
            setting.mode.clone(),
            "--pos".into(),
            format!("{},{}", setting.x, setting.y),
            "--scale".into(),
            setting.scale.to_string(),
        ]);
    }
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    crate::desktop::run("wlr-randr", &args)
        .await
        .context("Apply output configuration")?;
    Ok(())
}

async fn confirm(plan: &Plan) -> Result<layout::Layout> {
    for attempt in 0..5 {
        let current = layout::snapshot(&outputs().await?);
        ensure!(
            current.topology == plan.after.topology,
            "The connected outputs changed during preview"
        );
        if plan.after.matches(&current) {
            return Ok(current);
        }
        if attempt < 4 {
            tokio::time::sleep(Duration::from_millis(90)).await;
        }
    }
    anyhow::bail!("The compositor did not confirm the requested layout")
}

async fn restore(plan: &Plan) -> Result<()> {
    let observed = outputs().await?;
    let settings = plan.restoration(&observed);
    apply(&settings).await?;
    if settings.is_empty() {
        return Ok(());
    }
    let observed = layout::snapshot(&outputs().await?);
    ensure!(
        settings.iter().all(|setting| observed
            .settings
            .iter()
            .any(|actual| setting.matches(actual))),
        "An output could not be restored"
    );
    Ok(())
}

fn send(reply: Reply) -> Result<()> {
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, &reply)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

pub async fn run() -> Result<()> {
    let mut lines = BufReader::new(std::io::stdin().take(128 * 1024)).lines();
    let text = lines.next().context("Missing display plan")??;
    let job: Job = serde_json::from_str(&text)?;
    let observed = outputs().await?;
    ensure!(
        layout::snapshot(&observed).matches(&job.plan.before),
        "Outputs changed before the preview"
    );
    // Revalidate the job in the child before passing fixed arguments to wlr-randr.
    let plan = Plan::new(&observed, job.plan.after.settings.clone())?;
    ensure!(!plan.changes().is_empty(), "The layout is already active");
    let result = async {
        apply(&plan.after.settings).await?;
        confirm(&plan).await?;
        let expires_at = super::now_ms() + PREVIEW_MS;
        let (choice, receive) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let _ = choice.send(lines.next().and_then(Result::ok));
        });
        send(Reply::Applied {
            preview: Preview {
                token: job.token,
                expires_at,
            },
        })?;
        let keep = tokio::time::timeout(Duration::from_millis(PREVIEW_MS), receive)
            .await
            .ok()
            .and_then(Result::ok)
            .flatten()
            .is_some_and(|line| line == "keep");
        if keep {
            let current = confirm(&plan).await?;
            if job.automatic {
                return Ok("Remembered display layout restored.".into());
            }
            profiles::remember(&job.profile, current)?;
            Ok::<_, anyhow::Error>("Display layout saved.".into())
        } else {
            restore(&plan).await?;
            Ok("Previous display layout restored.".into())
        }
    }
    .await;
    let reply = match result {
        Ok(message) => Reply::Done {
            message,
            error: String::new(),
        },
        Err(error) => {
            let rollback = restore(&plan).await;
            Reply::Done {
                message: if rollback.is_ok() {
                    "Previous display layout restored."
                } else {
                    "Display restoration needs attention."
                }
                .into(),
                error: error.to_string(),
            }
        }
    };
    send(reply)
}
