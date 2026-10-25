use crate::poll::poll;
use async_channel::Receiver;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct BatteryState {
    pub percent: u8,
    pub charging: bool,
}

fn read() -> Option<BatteryState> {
    let dir = std::fs::read_dir("/sys/class/power_supply")
    .ok()?
    .flatten()
    .map(|e| e.path())
    .find(|p| {
        p.join("capacity").exists()
        && p.file_name()
        .and_then(|n| n.to_str())
        .map_or(false, |n| n.starts_with("BAT"))
    })?;
    let percent = std::fs::read_to_string(dir.join("capacity"))
    .ok()?
    .trim()
    .parse()
    .ok()?;
    let status = std::fs::read_to_string(dir.join("status")).unwrap_or_default();
    let charging = matches!(status.trim(), "Charging" | "Full");
    Some(BatteryState { percent, charging })
}

pub fn watch() -> Receiver<BatteryState> {
    poll(Duration::from_secs(15), read)
}
