use super::*;

#[derive(Default)]
pub(crate) struct DimLease {
    target: Option<SessionTarget>,
    lights: Vec<(String, u32, u32)>,
}
impl DimLease {
    pub(crate) fn active(&self) -> bool {
        !self.lights.is_empty()
    }
}

impl Controller {
    pub(crate) async fn idle_suspend(&self) -> Result<()> {
        let _permit = self
            .busy
            .clone()
            .try_acquire_owned()
            .context("A power action is already running")?;
        let backend = self.backend()?;
        let observed = self.refresh(backend.as_ref()).await?;
        let target = observed
            .session
            .target
            .context("No verified graphical session")?;
        let observed = self.prepare_sleep(backend.as_ref(), &target).await?;
        tokio::time::timeout(
            Duration::from_secs(15),
            backend.apply(
                Mutation::Session {
                    kind: SessionAction::Suspend,
                    target,
                },
                observed,
            ),
        )
        .await
        .context("Suspend request timed out")??;
        Ok(())
    }

    pub(crate) async fn dim(&self, percent: u8, lease: &mut DimLease) -> Result<()> {
        ensure!((5..=80).contains(&percent), "Invalid idle brightness");
        ensure!(
            !lease.active(),
            "Brightness is already leased by idle policy"
        );
        let _permit = self
            .busy
            .clone()
            .try_acquire_owned()
            .context("A power action is already running")?;
        let backend = self.backend()?;
        let observed = self.refresh(backend.as_ref()).await?;
        let target = observed
            .session
            .target
            .clone()
            .context("Dimming needs a verified graphical session")?;
        lease.target = Some(target.clone());
        for light in &observed.backlights {
            let dimmed = ((u64::from(light.max) * u64::from(percent) + 50) / 100).max(1) as u32;
            if dimmed >= light.value {
                continue;
            }
            let current = self.refresh(backend.as_ref()).await?;
            ensure!(
                current.session.target.as_ref() == Some(&target),
                "Graphical session changed before dimming"
            );
            if !current
                .backlights
                .iter()
                .any(|l| l.id == light.id && l.value == light.value)
            {
                continue;
            }
            // Record intent before dispatch. An uncertain reply can still have
            // changed the device; restoration checks the exact raw value.
            lease.lights.push((light.id.clone(), light.value, dimmed));
            tokio::time::timeout(
                Duration::from_secs(6),
                backend.apply(
                    Mutation::BrightnessDim {
                        device: light.id.clone(),
                        expected: light.value,
                        percent,
                    },
                    current,
                ),
            )
            .await
            .context("Idle dimming timed out")??;
        }
        Ok(())
    }

    pub(crate) async fn restore_brightness(&self, lease: &mut DimLease) -> Result<()> {
        if !lease.active() {
            return Ok(());
        }
        let _permit = tokio::time::timeout(Duration::from_secs(3), self.busy.acquire())
            .await
            .context("Brightness restoration is busy")??;
        let backend = self.backend()?;
        let mut first_error = None;
        for (device, original, dimmed) in &lease.lights {
            let result: Result<()> = async {
                let current = self.refresh(backend.as_ref()).await?;
                if current.session.target != lease.target {
                    return Ok(());
                }
                if !current
                    .backlights
                    .iter()
                    .any(|l| l.id == *device && l.value == *dimmed)
                {
                    return Ok(());
                }
                tokio::time::timeout(
                    Duration::from_secs(6),
                    backend.apply(
                        Mutation::BrightnessRestore {
                            device: device.clone(),
                            expected: *dimmed,
                            value: *original,
                        },
                        current,
                    ),
                )
                .await
                .context("Brightness restoration timed out")??;
                Ok(())
            }
            .await;
            if let Err(error) = result {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        *lease = DimLease::default();
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(())
    }
}
