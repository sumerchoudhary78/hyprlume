mod agent;
mod wifi;

use super::{
    pairing::Pairing, Backend, BluetoothAdapter, BluetoothDevice, BluetoothState, Hardware,
    Request, State, WifiState,
};
use anyhow::{bail, Context, Result};
use bluer::{AdapterEvent, AdapterProperty, DeviceProperty, Session};
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, SelectAll},
    StreamExt,
};
use nmrs::NetworkManager;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::mpsc,
    task::{JoinHandle, JoinSet},
};

pub(super) struct Native {
    network: Option<NetworkManager>,
    bluetooth: Option<Session>,
    pairing: Arc<Pairing>,
    changes: mpsc::Sender<()>,
    discovery: Mutex<HashMap<String, JoinHandle<()>>>,
    _watchers: JoinSet<()>,
}

impl Drop for Native {
    fn drop(&mut self) {
        for task in self.discovery.get_mut().unwrap().values() {
            task.abort();
        }
    }
}

impl Native {
    pub(super) async fn new(changes: mpsc::Sender<()>, pairing: Arc<Pairing>) -> Result<Self> {
        let (network, bluetooth) = tokio::join!(NetworkManager::new(), Session::new());
        anyhow::ensure!(
            network.is_ok() || bluetooth.is_ok(),
            "System D-Bus is unavailable"
        );
        let network = network.ok();
        let bluetooth = bluetooth.ok();
        let mut watchers = JoinSet::new();
        if let Some(manager) = network.clone() {
            let changes = changes.clone();
            watchers.spawn(async move {
                loop {
                    if let Ok(mut events) = manager.network_events().await {
                        while events.next().await.is_some() {
                            let _ = changes.try_send(());
                        }
                    }
                    let _ = changes.try_send(());
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        }
        if let Some(session) = bluetooth.clone() {
            let changes = changes.clone();
            watchers.spawn(async move {
                loop {
                    let _ = watch_bluetooth(&session, &changes).await;
                    let _ = changes.try_send(());
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        }
        Ok(Self {
            network,
            bluetooth,
            pairing,
            changes,
            discovery: Mutex::new(HashMap::new()),
            _watchers: watchers,
        })
    }

    async fn bluetooth_snapshot(&self) -> Result<BluetoothState> {
        let session = self.bluetooth.as_ref().context("BlueZ is unavailable")?;
        let mut adapters = Vec::new();
        let mut devices = Vec::new();
        for id in session.adapter_names().await?.into_iter().take(8) {
            let adapter = session.adapter(&id)?;
            let mut row = BluetoothAdapter {
                id: id.clone(),
                name: id.clone(),
                ..Default::default()
            };
            for property in adapter.all_properties().await? {
                match property {
                    AdapterProperty::Alias(name) => row.name = name,
                    AdapterProperty::Powered(powered) => row.powered = powered,
                    AdapterProperty::Discovering(discovering) => row.discovering = discovering,
                    _ => {}
                }
            }
            adapters.push(row);
            for address in adapter.device_addresses().await?.into_iter().take(128) {
                let device = adapter.device(address)?;
                let Ok(properties) = device.all_properties().await else {
                    continue;
                };
                let mut row = BluetoothDevice {
                    adapter: id.clone(),
                    address: address.to_string(),
                    name: address.to_string(),
                    ..Default::default()
                };
                for property in properties {
                    match property {
                        DeviceProperty::Alias(name) => row.name = name,
                        DeviceProperty::Icon(icon) => row.icon = icon,
                        DeviceProperty::Paired(paired) => row.paired = paired,
                        DeviceProperty::Connected(connected) => row.connected = connected,
                        DeviceProperty::Trusted(trusted) => row.trusted = trusted,
                        _ => {}
                    }
                }
                devices.push(row);
            }
        }
        Ok(BluetoothState {
            available: !adapters.is_empty(),
            error: if adapters.is_empty() {
                "No Bluetooth adapter found".into()
            } else {
                String::new()
            },
            adapters,
            devices,
        })
    }

    async fn bluetooth_execute(&self, request: Request) -> Result<()> {
        let session = self.bluetooth.as_ref().context("BlueZ is unavailable")?;
        match request {
            Request::BluetoothPower { adapter, enabled } => {
                let adapter = session.adapter(&adapter)?;
                adapter.set_powered(enabled).await?;
                anyhow::ensure!(
                    adapter.is_powered().await? == enabled,
                    "BlueZ did not confirm the radio state"
                );
            }
            Request::BluetoothScan { adapter: id } => {
                {
                    let scans = self.discovery.lock().unwrap();
                    if scans.get(&id).is_some_and(|task| !task.is_finished()) {
                        return Ok(());
                    }
                }
                let adapter = session.adapter(&id)?;
                let events = adapter.discover_devices_with_changes().await?;
                let changes = self.changes.clone();
                let task = tokio::spawn(async move {
                    tokio::pin!(events);
                    let deadline = tokio::time::sleep(Duration::from_secs(15));
                    tokio::pin!(deadline);
                    loop {
                        tokio::select! {
                            _ = &mut deadline => break,
                            event = events.next() => { if event.is_none() { break; } let _ = changes.try_send(()); }
                        }
                    }
                    let _ = changes.try_send(());
                });
                self.discovery.lock().unwrap().insert(id, task);
            }
            Request::BluetoothPair {
                adapter,
                address,
                trust,
            } => {
                let device = session.adapter(&adapter)?.device(address.parse()?)?;
                let _agent = session
                    .register_agent(agent::agent(self.pairing.clone()))
                    .await?;
                if !device.is_paired().await? {
                    device.pair().await?;
                }
                if trust {
                    device.set_trusted(true).await?;
                }
                device.connect().await?;
            }
            Request::BluetoothConnect { adapter, address } => {
                session
                    .adapter(&adapter)?
                    .device(address.parse()?)?
                    .connect()
                    .await?
            }
            Request::BluetoothDisconnect { adapter, address } => {
                session
                    .adapter(&adapter)?
                    .device(address.parse()?)?
                    .disconnect()
                    .await?
            }
            Request::BluetoothForget { adapter, address } => {
                session
                    .adapter(&adapter)?
                    .remove_device(address.parse()?)
                    .await?
            }
            _ => bail!("Not a Bluetooth operation"),
        }
        Ok(())
    }
}

impl Backend for Native {
    fn snapshot(&self) -> BoxFuture<'_, Result<Hardware>> {
        Box::pin(async move {
            let wifi = async {
                if let Some(manager) = &self.network {
                    wifi::snapshot(manager).await.unwrap_or_else(|_| WifiState {
                        error: "NetworkManager is unavailable".into(),
                        ..Default::default()
                    })
                } else {
                    WifiState {
                        error: "NetworkManager is unavailable".into(),
                        ..Default::default()
                    }
                }
            };
            let (wifi, bluetooth) = tokio::join!(wifi, self.bluetooth_snapshot());
            Ok(Hardware {
                wifi,
                bluetooth: bluetooth.unwrap_or_else(|_| BluetoothState {
                    error: "BlueZ is unavailable".into(),
                    ..Default::default()
                }),
            })
        })
    }

    fn execute(&self, request: Request, state: State) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            match request {
                Request::Refresh => Ok(()),
                Request::WifiScan { .. }
                | Request::WifiPower { .. }
                | Request::WifiConnect { .. }
                | Request::WifiDisconnect { .. }
                | Request::WifiForget { .. } => {
                    wifi::execute(
                        self.network
                            .as_ref()
                            .context("NetworkManager is unavailable")?,
                        request,
                        state,
                    )
                    .await
                }
                request => self.bluetooth_execute(request).await,
            }
        })
    }

    fn stop_discovery(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            for (_, task) in self.discovery.lock().unwrap().drain() {
                task.abort();
            }
        })
    }
}

async fn watch_bluetooth(session: &Session, changes: &mpsc::Sender<()>) -> Result<()> {
    loop {
        let mut streams: SelectAll<BoxStream<'static, bool>> = SelectAll::new();
        streams.push(session.events().await?.map(|_| true).boxed());
        for id in session.adapter_names().await?.into_iter().take(8) {
            let adapter = session.adapter(&id)?;
            streams.push(
                adapter
                    .events()
                    .await?
                    .map(|event| {
                        matches!(
                            event,
                            AdapterEvent::DeviceAdded(_) | AdapterEvent::DeviceRemoved(_)
                        )
                    })
                    .boxed(),
            );
            for address in adapter.device_addresses().await?.into_iter().take(128) {
                if let Ok(events) = adapter.device(address)?.events().await {
                    streams.push(events.map(|_| false).boxed());
                }
            }
        }
        loop {
            let rebuild = streams
                .next()
                .await
                .context("Bluetooth event streams ended")?;
            let _ = changes.try_send(());
            if rebuild {
                break;
            }
        }
    }
}
