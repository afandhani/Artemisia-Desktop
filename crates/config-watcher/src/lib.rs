//! Watches Artemisia configuration changes.

use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant},
};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use config::Config;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(250);

pub struct ConfigWatcher {
    // Harus tetap hidup agar watcher terus memantau file.
    _watcher: RecommendedWatcher,

    // Menerima event filesystem.
    receiver: Receiver<notify::Result<Event>>,

    // Lokasi config.toml.
    path: PathBuf,

    // Waktu event relevan terakhir diterima.
    pending_since: Option<Instant>,
}

impl ConfigWatcher {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = config::config_path();

        let parent = path.parent().ok_or("Config path has no parent directory")?;

        fs::create_dir_all(parent)?;

        let (sender, receiver) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = sender.send(event);
            },
            notify::Config::default(),
        )?;

        // Pantau direktori agar atomic file replacement terdeteksi.
        watcher.watch(parent, RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver,
            path,
            pending_since: None,
        })
    }

    /// Memeriksa event dan memuat ulang konfigurasi setelah debounce.
    ///
    /// Ok(Some(config)) = konfigurasi baru berhasil dimuat.
    /// Ok(None)         = belum waktunya reload atau tidak ada perubahan.
    /// Err(error)       = event atau konfigurasi mengalami masalah.
    pub fn poll(&mut self) -> Result<Option<Config>, Box<dyn Error>> {
        loop {
            match self.receiver.try_recv() {
                Ok(Ok(event)) => {
                    let relevant_kind = matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    );

                    let relevant_path = event
                        .paths
                        .iter()
                        .any(|event_path| event_path == &self.path);

                    if relevant_kind && relevant_path {
                        // Reset debounce setiap ada perubahan relevan.
                        self.pending_since = Some(Instant::now());
                    }
                }

                Ok(Err(error)) => {
                    return Err(Box::new(error));
                }

                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    return Err("Config watcher channel disconnected".into());
                }
            }
        }

        let ready = self
            .pending_since
            .map(|since| since.elapsed() >= DEBOUNCE_DURATION)
            .unwrap_or(false);

        if !ready {
            return Ok(None);
        }

        // Reset sebelum reload agar error tidak memicu percobaan berulang.
        self.pending_since = None;

        // Parsing, merging, dan validation dilakukan oleh config crate.
        let config = config::load()?;

        Ok(Some(config))
    }
}
