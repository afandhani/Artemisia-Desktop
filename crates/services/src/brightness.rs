use crate::poll::{poll, worker};
use async_channel::Receiver;
use std::process::Command;
use std::sync::{mpsc::Sender, OnceLock};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct BrightnessState {
    pub percent: u8,
}

fn read() -> Option<BrightnessState> {
    let out = Command::new("brightnessctl")
    .args(["-c", "backlight", "-m"])
    .output()
    .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // contoh: "intel_backlight,backlight,12000,50%,24000"
    let percent = text
    .lines()
    .next()?
    .split(',')
    .nth(3)?
    .trim_end_matches('%')
    .parse()
    .ok()?;
    Some(BrightnessState { percent })
}

pub fn watch() -> Receiver<BrightnessState> {
    poll(Duration::from_secs(2), read)
}

static SETTER: OnceLock<Sender<u8>> = OnceLock::new();

pub fn set(percent: u8) {
    let tx = SETTER.get_or_init(|| {
        worker(|p| {
            let _ = Command::new("brightnessctl")
            .args(["-c", "backlight", "set", &format!("{p}%")])
            .status();
        })
    });
    // minimal 1% supaya layar tidak mati total
    let _ = tx.send(percent.clamp(1, 100));
}
