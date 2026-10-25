use crate::poll::poll;
use async_channel::Receiver;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkState {
    Disconnected,
    Wifi { ssid: String, signal: u8 },
}

fn read() -> Option<NetworkState> {
    let out = Command::new("nmcli")
    .args(["-t", "-f", "IN-USE,SIGNAL,SSID", "dev", "wifi", "list", "--rescan", "no"])
    .output()
    .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let mut parts = line.splitn(3, ':');
        if parts.next()? == "*" {
            let signal = parts.next()?.parse().ok()?;
            let ssid = parts.next()?.replace("\\:", ":");
            return Some(NetworkState::Wifi { ssid, signal });
        }
    }
    Some(NetworkState::Disconnected)
}

pub fn watch() -> Receiver<NetworkState> {
    poll(Duration::from_secs(5), read)
}
