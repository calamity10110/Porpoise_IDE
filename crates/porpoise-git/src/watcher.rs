use std::{path::Path, sync::mpsc};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use porpoise_core::error::{PorpoiseError, Result};

use crate::types::{WatchEvent, WatchEventKind};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<notify::Result<notify::Event>>,
}

impl FileWatcher {
    pub fn watch(path: &Path) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| PorpoiseError::Runtime(format!("file watcher: {e}")))?;
        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| PorpoiseError::Runtime(format!("watch {path:?}: {e}")))?;
        Ok(Self { _watcher: watcher, rx })
    }

    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.rx.try_recv().ok().and_then(|result| {
            let ev = result.ok()?;
            let kind = match ev.kind {
                notify::EventKind::Create(_) => WatchEventKind::Create,
                notify::EventKind::Modify(_) => WatchEventKind::Modify,
                notify::EventKind::Remove(_) => WatchEventKind::Delete,
                _ => return None,
            };
            let path = ev.paths.into_iter().next()?;
            Some(WatchEvent { path, kind })
        })
    }
}
