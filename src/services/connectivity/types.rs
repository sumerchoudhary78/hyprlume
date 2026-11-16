use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

/// Transport-only credential. Never put this value in a snapshot or receipt.
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl Drop for Secret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Toggle,
    Close,
    Refresh,
    WifiScan {
        interface: Option<String>,
    },
    WifiPower {
        enabled: bool,
    },
    WifiConnect {
        network: String,
        password: Option<Secret>,
    },
    WifiDisconnect {
        interface: String,
    },
    WifiForget {
        uuid: String,
    },
    BluetoothScan {
        adapter: String,
    },
    BluetoothPower {
        adapter: String,
        enabled: bool,
    },
    BluetoothPair {
        adapter: String,
        address: String,
        #[serde(default)]
        trust: bool,
    },
    BluetoothConnect {
        adapter: String,
        address: String,
    },
    BluetoothDisconnect {
        adapter: String,
        address: String,
    },
    BluetoothForget {
        adapter: String,
        address: String,
    },
    PairReply {
        token: u64,
        accept: bool,
        value: Option<Secret>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WifiInterface {
    pub name: String,
    pub connected: bool,
    pub ssid: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub id: String,
    pub interface: String,
    pub ssid: String,
    pub bssid: String,
    pub strength: u8,
    pub security: String,
    pub connected: bool,
    pub saved_uuid: Option<String>,
    pub frequency: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WifiState {
    pub available: bool,
    pub enabled: bool,
    pub hardware_enabled: bool,
    pub error: String,
    pub interfaces: Vec<WifiInterface>,
    pub networks: Vec<WifiNetwork>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BluetoothAdapter {
    pub id: String,
    pub name: String,
    pub powered: bool,
    pub discovering: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub adapter: String,
    pub address: String,
    pub name: String,
    pub icon: String,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BluetoothState {
    pub available: bool,
    pub error: String,
    pub adapters: Vec<BluetoothAdapter>,
    pub devices: Vec<BluetoothDevice>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hardware {
    pub wifi: WifiState,
    pub bluetooth: BluetoothState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairPrompt {
    pub token: u64,
    pub adapter: String,
    pub address: String,
    pub kind: String,
    pub code: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub panel_open: bool,
    pub busy: bool,
    pub message: String,
    pub error: String,
    #[serde(flatten)]
    pub hardware: Hardware,
    pub prompt: Option<PairPrompt>,
}

impl Request {
    pub(crate) fn validate(&self, state: &State) -> Result<()> {
        let wifi_interface = |name: &str| {
            state
                .hardware
                .wifi
                .interfaces
                .iter()
                .any(|i| i.name == name)
        };
        let bluetooth_adapter =
            |id: &str| state.hardware.bluetooth.adapters.iter().any(|a| a.id == id);
        let bluetooth_device = |adapter: &str, address: &str| {
            state
                .hardware
                .bluetooth
                .devices
                .iter()
                .any(|d| d.adapter == adapter && d.address == address)
        };
        match self {
            Self::WifiScan {
                interface: Some(name),
            }
            | Self::WifiDisconnect { interface: name }
                if !wifi_interface(name) =>
            {
                bail!("Wi-Fi interface is no longer available")
            }
            Self::WifiConnect { network, password } => {
                anyhow::ensure!(
                    state
                        .hardware
                        .wifi
                        .networks
                        .iter()
                        .any(|n| n.id == *network),
                    "Network is no longer in the current scan"
                );
                if let Some(password) = password {
                    anyhow::ensure!(
                        password.expose().len() <= 128 && !password.expose().contains('\0'),
                        "Invalid Wi-Fi credential"
                    );
                }
            }
            Self::WifiForget { uuid }
                if !state
                    .hardware
                    .wifi
                    .networks
                    .iter()
                    .any(|n| n.saved_uuid.as_deref() == Some(uuid)) =>
            {
                bail!("Saved network is no longer available")
            }
            Self::BluetoothScan { adapter } | Self::BluetoothPower { adapter, .. }
                if !bluetooth_adapter(adapter) =>
            {
                bail!("Bluetooth adapter is no longer available")
            }
            Self::BluetoothPair {
                adapter, address, ..
            }
            | Self::BluetoothConnect { adapter, address }
            | Self::BluetoothDisconnect { adapter, address }
            | Self::BluetoothForget { adapter, address }
                if !bluetooth_device(adapter, address) =>
            {
                bail!("Bluetooth device is no longer available")
            }
            Self::PairReply {
                value: Some(value), ..
            } if value.expose().len() > 16 || value.expose().chars().any(char::is_control) => {
                bail!("Invalid pairing response")
            }
            _ => {}
        }
        Ok(())
    }
}

impl Hardware {
    pub(crate) fn bounded(mut self) -> Self {
        let identifier = |value: &str, limit: usize| {
            value.len() <= limit && value.is_ascii() && !value.chars().any(char::is_control)
        };
        self.wifi.error = self.wifi.error.chars().take(256).collect();
        self.bluetooth.error = self.bluetooth.error.chars().take(256).collect();
        self.wifi
            .interfaces
            .retain(|item| identifier(&item.name, 32));
        for interface in &mut self.wifi.interfaces {
            interface.ssid = interface.ssid.chars().take(32).collect();
        }
        self.wifi.interfaces.truncate(8);
        self.wifi.networks.retain(|network| {
            identifier(&network.id, 128)
                && identifier(&network.interface, 32)
                && identifier(&network.bssid, 17)
                && identifier(&network.security, 16)
                && network.ssid.len() <= 96
                && network
                    .saved_uuid
                    .as_ref()
                    .is_none_or(|uuid| identifier(uuid, 64))
        });
        self.wifi
            .networks
            .sort_by_key(|n| (!n.connected, std::cmp::Reverse(n.strength), n.ssid.clone()));
        self.wifi.networks.truncate(32);
        self.bluetooth
            .adapters
            .retain(|adapter| identifier(&adapter.id, 32));
        for adapter in &mut self.bluetooth.adapters {
            adapter.name = adapter.name.chars().take(64).collect();
        }
        self.bluetooth.adapters.truncate(8);
        self.bluetooth
            .devices
            .retain(|device| identifier(&device.adapter, 32) && identifier(&device.address, 17));
        self.bluetooth
            .devices
            .sort_by_key(|d| (!d.connected, !d.paired, d.name.clone()));
        self.bluetooth.devices.truncate(32);
        for device in &mut self.bluetooth.devices {
            device.name = device.name.chars().take(64).collect();
            device.icon = device
                .icon
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
                .take(48)
                .collect();
        }
        self
    }
}
