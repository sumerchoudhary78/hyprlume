use std::{process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use zbus::Connection;

pub(crate) struct Bus {
    pub(crate) address: String,
    _child: Child,
    _directory: tempfile::TempDir,
}
impl Bus {
    pub(crate) async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("bus.conf");
        std::fs::write(&config, format!("<busconfig><type>session</type><listen>unix:tmpdir={}</listen><auth>EXTERNAL</auth><policy context=\"default\"><allow send_destination=\"*\"/><allow receive_sender=\"*\"/><allow own=\"*\"/></policy></busconfig>", directory.path().display())).unwrap();
        let mut child = Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("install dbus-daemon for isolated desktop service tests");
        let mut address = String::new();
        tokio::time::timeout(
            Duration::from_secs(3),
            BufReader::new(child.stdout.take().unwrap()).read_line(&mut address),
        )
        .await
        .unwrap()
        .unwrap();
        Self {
            address: address.trim().into(),
            _child: child,
            _directory: directory,
        }
    }
    pub(crate) async fn connect(&self) -> Connection {
        zbus::connection::Builder::address(self.address.as_str())
            .unwrap()
            .build()
            .await
            .unwrap()
    }
}
