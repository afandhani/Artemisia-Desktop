use serde::Deserialize;
use std::io::BufReader;
use std::process::{Command, Stdio};

pub use async_channel::Receiver;

// Tipe publik: hanya ini yang dikenal UI.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub index: u8,
    pub active: bool,
    pub urgent: bool,
    pub occupied: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TagsState {
    pub tags: Vec<Tag>,
}

// Format mmsg: privat, hanya modul ini yang tahu.
#[derive(Deserialize)]
struct RawAll {
    all_tags: Vec<RawMonitor>,
}
#[derive(Deserialize)]
struct RawMonitor {
    tags: Vec<RawTag>,
}
#[derive(Deserialize)]
struct RawTag {
    index: u8,
    is_active: bool,
    is_urgent: bool,
    client_count: u32,
}

fn convert(raw: RawAll) -> Option<TagsState> {
    // sementara: monitor pertama saja
    let mon = raw.all_tags.into_iter().next()?;
    let tags = mon
    .tags
    .into_iter()
    .map(|t| Tag {
        index: t.index,
         active: t.is_active,
         urgent: t.is_urgent,
         occupied: t.client_count > 0,
    })
    .collect();
    Some(TagsState { tags })
}

fn fetch() -> Option<TagsState> {
    let out = Command::new("mmsg").args(["get", "all-tags"]).output().ok()?;
    let raw: RawAll = serde_json::from_slice(&out.stdout).ok()?;
    convert(raw)
}

/// Mulai memantau tag; state baru dikirim lewat Receiver.
pub fn watch() -> Receiver<TagsState> {
    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(move || {
        let mut last: Option<TagsState> = None;

        if let Some(state) = fetch() {
            if tx.send_blocking(state.clone()).is_err() {
                return;
            }
            last = Some(state);
        }

        let mut child = match Command::new("mmsg")
        .args(["watch", "all-tags"])
        .stdout(Stdio::piped())
        .spawn()
        {
            Ok(c) => c,
                       Err(e) => {
                           eprintln!("mango: gagal menjalankan mmsg: {e}");
                           return;
                       }
        };

        let stdout = child.stdout.take().expect("stdout piped");
        let stream = serde_json::Deserializer::from_reader(BufReader::new(stdout))
        .into_iter::<serde_json::Value>();

        for value in stream {
            let Ok(value) = value else { break };
            match serde_json::from_value::<RawAll>(value.clone()) {
                Ok(raw) => {
                    if let Some(state) = convert(raw) {
                        if last.as_ref() == Some(&state) {
                            continue;
                        }
                        if tx.send_blocking(state.clone()).is_err() {
                            break;
                        }
                        last = Some(state);
                    }
                }
                Err(e) => eprintln!("mango: format tidak dikenal ({e}): {value}"),
            }
        }
        let _ = child.kill();
    });

    rx
}

/// Pindah ke tag tertentu (1-9).
pub fn view_tag(index: u8) {
    std::thread::spawn(move || {
        let out = Command::new("mmsg")
        .args(["dispatch", &format!("view,{index}")])
        .output();
        eprintln!("mango: view,{index} -> {out:?}");
    });
}
