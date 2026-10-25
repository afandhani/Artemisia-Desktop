use crate::poll::poll;
use async_channel::Receiver;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct VolumeState {
    pub percent: u8,
    pub muted: bool,
}

fn pactl(args: &[&str]) -> Option<String> {
    let out = Command::new("pactl").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn read() -> Option<VolumeState> {
    // contoh: "Volume: front-left: 32768 /  50% / -18.06 dB, front-right: ..."
    let vol = pactl(&["get-sink-volume", "@DEFAULT_SINK@"])?;
    let percent = vol
    .split_whitespace()
    .find(|t| t.ends_with('%'))?
    .trim_end_matches('%')
    .parse()
    .ok()?;
    // contoh: "Mute: no"
    let muted = pactl(&["get-sink-mute", "@DEFAULT_SINK@"])
    .map_or(false, |m| m.contains("yes"));
    Some(VolumeState { percent, muted })
}

pub fn watch() -> Receiver<VolumeState> {
    poll(Duration::from_secs(1), read)
}
