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

// ---- daftar wifi untuk control center ----

#[derive(Debug, Clone, PartialEq)]
pub struct AccessPoint {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WifiList {
    pub enabled: bool,
    pub saved: Vec<String>,
    pub aps: Vec<AccessPoint>,
}

fn nmcli(args: &[&str]) -> Option<String> {
    let out = Command::new("nmcli").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn read_list() -> Option<WifiList> {
    let enabled = nmcli(&["radio", "wifi"])?.trim() == "enabled";
    if !enabled {
        return Some(WifiList { enabled, saved: vec![], aps: vec![] });
    }

    let saved: Vec<String> = nmcli(&["-t", "-f", "NAME", "connection", "show"])
    .unwrap_or_default()
    .lines()
    .map(|l| l.replace("\\:", ":"))
    .collect();

    let text = nmcli(&[
        "-t", "-f", "IN-USE,SIGNAL,SECURITY,SSID", "dev", "wifi", "list", "--rescan", "no",
    ])?;

    let mut aps: Vec<AccessPoint> = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.splitn(4, ':').collect();
        if f.len() < 4 || f[3].is_empty() {
            continue;
        }
        let ap = AccessPoint {
            ssid: f[3].replace("\\:", ":"),
            signal: f[1].parse().unwrap_or(0),
            secured: !f[2].is_empty() && f[2] != "--",
            active: f[0] == "*",
        };
        // SSID sama di beberapa band: ambil yang aktif / terkuat.
        match aps.iter_mut().find(|a| a.ssid == ap.ssid) {
            Some(existing) => {
                if ap.active || (!existing.active && ap.signal > existing.signal) {
                    *existing = ap;
                }
            }
            None => aps.push(ap),
        }
    }
    aps.sort_by(|a, b| b.active.cmp(&a.active).then(b.signal.cmp(&a.signal)));

    Some(WifiList { enabled, saved, aps })
}

pub fn watch_list() -> Receiver<WifiList> {
    poll(Duration::from_secs(4), read_list)
}

fn run_detached(args: Vec<String>) {
    std::thread::spawn(move || {
        let _ = Command::new("nmcli").args(args).status();
    });
}

pub fn set_wifi_enabled(on: bool) {
    let state = if on { "on" } else { "off" };
    run_detached(vec!["radio".into(), "wifi".into(), state.into()]);
}

pub fn rescan() {
    run_detached(vec!["device".into(), "wifi".into(), "rescan".into()]);
}

/// Sambung ke jaringan baru (terbuka, atau terkunci dengan password).
pub fn connect(ssid: &str, password: Option<&str>) {
    let mut args = vec![
        "device".to_string(),
        "wifi".to_string(),
        "connect".to_string(),
        ssid.to_string(),
    ];
    if let Some(pw) = password {
        args.push("password".into());
        args.push(pw.to_string());
    }
    run_detached(args);
}

/// Sambung ke jaringan yang profilnya sudah tersimpan.
pub fn connect_saved(ssid: &str) {
    run_detached(vec!["connection".into(), "up".into(), "id".into(), ssid.to_string()]);
}

pub fn disconnect(ssid: &str) {
    run_detached(vec!["connection".into(), "down".into(), "id".into(), ssid.to_string()]);
}
