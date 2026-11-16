use super::super::{Request, State, WifiInterface, WifiNetwork, WifiState};
use anyhow::{bail, Context, Result};
use nmrs::{ConnectByUuidConfig, NetworkManager, WifiSecurity};
use std::fmt::Write;

fn network_id(interface: &str, ssid: &[u8], security: &str) -> String {
    let mut id = String::with_capacity(interface.len() + 1 + ssid.len() * 2);
    id.push_str(interface);
    id.push(':');
    for byte in ssid {
        write!(&mut id, "{byte:02x}").expect("String formatting is infallible");
    }
    id.push(':');
    id.push_str(security);
    id
}

pub(super) async fn snapshot(manager: &NetworkManager) -> Result<WifiState> {
    let snapshot = manager.snapshot().await?;
    let networks = snapshot
        .wifi_groups()
        .into_iter()
        .filter(|group| !group.strongest.is_hidden())
        .map(|group| {
            let access = group.strongest;
            let security = if access.security.eap {
                "enterprise"
            } else if access.security.sae {
                "wpa3"
            } else if access.security.psk {
                "wpa2"
            } else if access.security.is_open() {
                "open"
            } else {
                "unsupported"
            };
            WifiNetwork {
                id: network_id(&group.interface, &access.ssid_bytes, security),
                interface: group.interface,
                ssid: group.ssid,
                bssid: access.bssid,
                strength: access.strength,
                security: security.into(),
                connected: group.active,
                saved_uuid: group
                    .saved_profiles
                    .first()
                    .map(|profile| profile.uuid.clone()),
                frequency: access.frequency_mhz,
            }
        })
        .collect();
    Ok(WifiState {
        available: snapshot.wifi.present,
        enabled: snapshot.wifi.enabled,
        hardware_enabled: snapshot.wifi.hardware_enabled,
        error: String::new(),
        networks,
        interfaces: snapshot
            .wifi_devices
            .into_iter()
            .map(|device| WifiInterface {
                name: device.interface,
                connected: device.is_active,
                ssid: device.active_ssid.unwrap_or_default(),
            })
            .collect(),
    })
}

pub(super) async fn execute(
    manager: &NetworkManager,
    request: Request,
    state: State,
) -> Result<()> {
    match request {
        Request::WifiScan { interface } => manager.scan_networks(interface.as_deref()).await?,
        Request::WifiPower { enabled } => {
            if enabled {
                anyhow::ensure!(
                    state.hardware.wifi.hardware_enabled,
                    "Wi-Fi is blocked by the hardware switch"
                );
            }
            manager.set_wireless_enabled(enabled).await?;
            anyhow::ensure!(
                manager.wifi_state().await?.enabled == enabled,
                "NetworkManager did not confirm the radio state"
            );
        }
        Request::WifiDisconnect { interface } => manager.disconnect(Some(&interface)).await?,
        Request::WifiForget { uuid } => manager.delete_saved_connection(&uuid).await?,
        Request::WifiConnect { network, password } => {
            let network = state
                .hardware
                .wifi
                .networks
                .iter()
                .find(|n| n.id == network)
                .context("Network is no longer available")?;
            if let Some(uuid) = network.saved_uuid.as_ref().filter(|_| password.is_none()) {
                manager
                    .connect_by_uuid(
                        uuid,
                        ConnectByUuidConfig::default().with_interface(&network.interface),
                    )
                    .await?;
            } else {
                let security = match network.security.as_str() {
                    "open" => WifiSecurity::Open,
                    "wpa2" | "wpa3" => {
                        let psk = password
                            .as_ref()
                            .context("Enter this network's password")?
                            .expose();
                        let valid = if network.security == "wpa3" {
                            !psk.is_empty() && psk.len() <= 63
                        } else {
                            (8..=63).contains(&psk.len())
                                || psk.len() == 64 && psk.bytes().all(|ch| ch.is_ascii_hexdigit())
                        };
                        anyhow::ensure!(valid, "Invalid password length for this network");
                        if network.security == "wpa3" {
                            WifiSecurity::Sae { psk: psk.into() }
                        } else {
                            WifiSecurity::WpaPsk { psk: psk.into() }
                        }
                    }
                    "enterprise" => bail!(
                        "Configure an 802.1X profile in NetworkManager first, then select its saved connection"
                    ),
                    _ => bail!("This access point uses an unsupported security mode"),
                };
                manager
                    .connect_to_bssid(
                        &network.ssid,
                        Some(&network.bssid),
                        Some(&network.interface),
                        security,
                    )
                    .await?;
            }
        }
        _ => bail!("Not a Wi-Fi operation"),
    }
    Ok(())
}
