use crate::poll::{poll, worker};
use async_channel::Receiver;
use std::process::Command;
use std::sync::{mpsc::Sender, OnceLock};
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
    let vol = pactl(&["get-sink-volume", "@DEFAULT_SINK@"])?;
    let percent = vol
    .split_whitespace()
    .find(|t| t.ends_with('%'))?
    .trim_end_matches('%')
    .parse()
    .ok()?;
    let muted = pactl(&["get-sink-mute", "@DEFAULT_SINK@"])
    .map_or(false, |m| m.contains("yes"));
    Some(VolumeState { percent, muted })
}

pub fn watch() -> Receiver<VolumeState> {
    poll(Duration::from_secs(1), read)
}

static SETTER: OnceLock<Sender<u8>> = OnceLock::new();

pub fn set(percent: u8) {
    let tx = SETTER.get_or_init(|| {
        worker(|p| {
            let _ = Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{p}%")])
            .status();
        })
    });
    let _ = tx.send(percent.min(100));
}

pub fn toggle_mute() {
    std::thread::spawn(|| {
        let _ = Command::new("pactl")
        .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        .status();
    });
}
