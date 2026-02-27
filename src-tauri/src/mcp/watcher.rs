use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::time::sleep;
use crate::error::Result;

use crate::constants::timing::{WATCHER_DEBOUNCE, WATCHER_POLL_INTERVAL};

#[derive(Debug)]
pub struct WatcherManager {
    watcher: Option<RecommendedWatcher>,
    watched_paths: HashSet<PathBuf>,
}

impl WatcherManager {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_paths: HashSet::new(),
        }
    }

    pub fn start<F>(&mut self, paths: Vec<PathBuf>, mut on_event: F) -> Result<()> 
    where 
        F: FnMut() + Send + 'static 
    {
        // Stop any existing watcher
        self.stop();

        if paths.is_empty() {
            return Ok(());
        }

        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let mut ignored = false;
                    for path in &event.paths {
                        if path.components().any(|c| {
                            let s = c.as_os_str().to_string_lossy();
                            matches!(s.as_ref(), ".git" | "node_modules" | "target" | ".agents")
                        }) {
                            ignored = true;
                            break;
                        }
                    }
                    if !ignored {
                        let _ = tx.blocking_send(event);
                    }
                }
            },
            Config::default(),
        ).map_err(|e| crate::error::AppError::Watcher { message: format!("Failed to create watcher: {}", e) })?;

        for path in &paths {
            if path.exists() {
                let canonical_path = match std::fs::canonicalize(path) {
                    Ok(p) => p,
                    Err(e) => {
                        log::warn!("Failed to canonicalize path '{}': {}. Using original path.", path.display(), e);
                        path.clone()
                    }
                };
                if let Err(e) = watcher.watch(&canonical_path, RecursiveMode::Recursive) {
                    log::warn!("Failed to watch path '{}': {}", canonical_path.display(), e);
                } else {
                    self.watched_paths.insert(canonical_path);
                }
            }
        }

        self.watcher = Some(watcher);

        // Spawn debouncer task
        tokio::spawn(async move {
            let mut last_event = None;

            loop {
                tokio::select! {
                    maybe_event = rx.recv() => {
                        match maybe_event {
                            Some(_event) => {
                                last_event = Some(tokio::time::Instant::now());
                            }
                            None => {
                                // Channel closed, the watcher has been dropped.
                                break;
                            }
                        }
                    }
                    _ = sleep(WATCHER_POLL_INTERVAL) => {
                        if let Some(instant) = last_event {
                            if instant.elapsed() >= WATCHER_DEBOUNCE {
                                on_event();
                                last_event = None;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        self.watcher = None;
        self.watched_paths.clear();
    }

    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_watcher_lifecycle() {
        let mut manager = WatcherManager::new();
        assert!(!manager.is_watching());

        let temp_dir = TempDir::new().unwrap();
        manager.start(vec![temp_dir.path().to_path_buf()], || {}).unwrap();
        assert!(manager.is_watching());

        manager.stop();
        assert!(!manager.is_watching());
    }

    #[tokio::test]
    async fn test_watcher_debounce() {
        let mut manager = WatcherManager::new();
        let temp_dir = TempDir::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        manager.start(vec![temp_dir.path().to_path_buf()], move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        let file_path = temp_dir.path().join("test.txt");

        // Perform multiple writes
        for i in 0..5 {
            fs::write(&file_path, format!("content {}", i)).unwrap();
            sleep(Duration::from_millis(50)).await;
        }

        // Use a retry loop with timeout to avoid flaky timing assumptions
        let start = tokio::time::Instant::now();
        let timeout = Duration::from_secs(5);
        let mut triggered = false;

        while start.elapsed() < timeout {
            if counter.load(Ordering::SeqCst) == 1 {
                triggered = true;
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        assert!(triggered, "Should have triggered exactly once due to debounce within timeout");
        
        // Wait a bit more to ensure no additional triggers happen
        sleep(Duration::from_millis(500)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1, "Should not have triggered more than once");

        manager.stop();
    }
}
