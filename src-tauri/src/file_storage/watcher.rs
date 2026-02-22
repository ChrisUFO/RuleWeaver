#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{AppError, Result};

pub type FileChangeCallback = Box<dyn Fn(FileChangeEvent) + Send + 'static>;

#[derive(Debug, Clone)]
pub enum FileChangeEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

#[derive(Debug, Clone)]
pub struct RuleFileWatcher {
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    is_running: Arc<Mutex<bool>>,
    watched_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl RuleFileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(false)),
            watched_paths: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self, path: &std::path::Path, callback: FileChangeCallback) -> Result<()> {
        let mut is_running = self.is_running.lock().map_err(|_| AppError::LockError)?;
        if *is_running {
            return Ok(());
        }

        let (tx, rx): (
            Sender<Result<FileChangeEvent>>,
            Receiver<Result<FileChangeEvent>>,
        ) = channel();

        let event_callback = callback;
        let callback_arc = Arc::new(Mutex::new(event_callback));

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let event = match res {
                    Ok(e) => e,
                    Err(e) => {
                        let _ = tx.send(Err(AppError::InvalidInput {
                            message: format!("Watch error: {}", e),
                        }));
                        return;
                    }
                };

                for path in &event.paths {
                    if path.extension().and_then(|e| e.to_str()) != Some("md") {
                        continue;
                    }

                    let file_event = if event.kind.is_create() {
                        Some(FileChangeEvent::Created(path.clone()))
                    } else if event.kind.is_modify() {
                        Some(FileChangeEvent::Modified(path.clone()))
                    } else if event.kind.is_remove() {
                        Some(FileChangeEvent::Deleted(path.clone()))
                    } else {
                        None
                    };

                    if let Some(fe) = file_event {
                        let _ = tx.send(Ok(fe));
                    }
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_millis(500))
                .with_compare_contents(false),
        )
        .map_err(|e| AppError::InvalidInput {
            message: format!("Failed to create file watcher: {}", e),
        })?;

        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| AppError::InvalidInput {
                message: format!("Failed to watch path '{}': {}", path.display(), e),
            })?;

        {
            let mut w = self.watcher.lock().map_err(|_| AppError::LockError)?;
            *w = Some(watcher);
        }

        {
            let mut watched = self.watched_paths.lock().map_err(|_| AppError::LockError)?;
            watched.push(path.to_path_buf());
        }

        *is_running = true;
        drop(is_running);

        let is_running_clone = Arc::clone(&self.is_running);
        let callback_clone = Arc::clone(&callback_arc);

        thread::spawn(move || {
            while let Ok(event_result) = rx.recv() {
                let running = is_running_clone.lock().map(|g| *g).unwrap_or(false);
                if !running {
                    break;
                }

                if let Ok(event) = event_result {
                    if let Ok(cb) = callback_clone.lock() {
                        cb(event);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().map_err(|_| AppError::LockError)?;
        if !*is_running {
            return Ok(());
        }

        let mut watcher_guard = self.watcher.lock().map_err(|_| AppError::LockError)?;

        if let Some(mut watcher) = watcher_guard.take() {
            let watched = self.watched_paths.lock().map_err(|_| AppError::LockError)?;
            for path in watched.iter() {
                let _ = watcher.unwatch(path);
            }
        }

        *is_running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|g| *g).unwrap_or(false)
    }

    pub fn watched_paths(&self) -> Vec<PathBuf> {
        self.watched_paths
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}

impl Default for RuleFileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[test]
    fn test_watcher_creation() {
        let watcher = RuleFileWatcher::new();
        assert!(!watcher.is_running());
        assert!(watcher.watched_paths().is_empty());
    }

    #[test]
    fn test_watcher_start_and_stop() {
        let temp_dir = std::env::temp_dir().join(format!("watcher_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let watcher = RuleFileWatcher::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let callback = Box::new(move |_event: FileChangeEvent| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let result = watcher.start(&temp_dir, callback);
        assert!(result.is_ok());
        assert!(watcher.is_running());
        assert!(!watcher.watched_paths().is_empty());

        thread::sleep(Duration::from_millis(100));

        let result = watcher.stop();
        assert!(result.is_ok());
        assert!(!watcher.is_running());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
