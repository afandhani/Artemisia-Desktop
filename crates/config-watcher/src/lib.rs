//! Watches Artemisia configuration changes.

use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use config::Config;

pub struct ConfigWatcher {
    // Harus tetap hidup agar watcher terus memantau file.
    _watcher: RecommendedWatcher,

    // Menerima event filesystem.
    receiver: Receiver<notify::Result<Event>>,

    // Lokasi config.toml.
    path: PathBuf,
}

impl ConfigWatcher {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = config::config_path();

        let parent = path.parent().ok_or("Config path has no parent directory")?;

        // Pastikan direktori konfigurasi tersedia.
        fs::create_dir_all(parent)?;

        let (sender, receiver) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = sender.send(event);
            },
            notify::Config::default(),
        )?;

        // Pantau direktori, bukan hanya file.
        // Ini juga membantu mendeteksi atomic file replacement.
        watcher.watch(parent, RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver,
            path,
        })
    }

    /// Periksa event filesystem dan muat ulang config jika berubah.
    ///
    /// Ok(Some(config)) = konfigurasi baru berhasil dimuat.
    /// Ok(None)         = belum ada perubahan relevan.
    /// Err(error)       = event atau konfigurasi mengalami masalah.
    pub fn poll(&self) -> Result<Option<Config>, Box<dyn Error>> {
        let mut changed = false;

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
                        changed = true;
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

        if !changed {
            return Ok(None);
        }

        // Config::load() sudah melakukan parsing, merge, dan validation.
        let config: Config = config::load()?;

        Ok(Some(config))
    }
}
