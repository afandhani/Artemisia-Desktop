use crate::poll::poll;
use async_channel::Receiver;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct BtDevice {
    pub mac: String,
    pub name: String,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BtState {
    pub available: bool,
    pub powered: bool,
    pub devices: Vec<BtDevice>,
}

fn ctl(args: &[&str]) -> Option<String> {
    let out = Command::new("bluetoothctl").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

// Baris: "Device AA:BB:CC:DD:EE:FF Nama Perangkat"
fn parse_devices(text: &str) -> Vec<(String, String)> {
    text.lines()
    .filter_map(|l| {
        let rest = l.trim().strip_prefix("Device ")?;
        let (mac, name) = rest.split_once(' ')?;
        Some((mac.to_string(), name.trim().to_string()))
    })
    .collect()
}

fn read() -> Option<BtState> {
    let unavailable = BtState { available: false, powered: false, devices: vec![] };

    let Some(show) = ctl(&["show"]) else {
        return Some(unavailable);
    };
    if !show.contains("Powered:") {
        return Some(unavailable);
    }
    let powered = show.lines().any(|l| l.trim() == "Powered: yes");

    let connected: Vec<String> = parse_devices(&ctl(&["devices", "Connected"]).unwrap_or_default())
    .into_iter()
    .map(|(mac, _)| mac)
    .collect();

    let mut devices: Vec<BtDevice> = parse_devices(&ctl(&["devices", "Paired"]).unwrap_or_default())
    .into_iter()
    .map(|(mac, name)| BtDevice {
        connected: connected.contains(&mac),
         mac,
         name,
    })
    .collect();
    devices.sort_by(|a, b| b.connected.cmp(&a.connected).then(a.name.cmp(&b.name)));

    Some(BtState { available: true, powered, devices })
}

pub fn watch() -> Receiver<BtState> {
    poll(Duration::from_secs(3), read)
}

fn run_detached(args: Vec<String>) {
    std::thread::spawn(move || {
        let _ = Command::new("bluetoothctl").args(args).status();
    });
}

pub fn set_powered(on: bool) {
    let state = if on { "on" } else { "off" };
    run_detached(vec!["power".into(), state.into()]);
}

pub fn connect(mac: &str) {
    run_detached(vec!["connect".into(), mac.to_string()]);
}

pub fn disconnect(mac: &str) {
    run_detached(vec!["disconnect".into(), mac.to_string()]);
}
