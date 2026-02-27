use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use crate::error::Result;

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
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        ).map_err(|e| crate::error::AppError::Watcher { message: format!("Failed to create watcher: {}", e) })?;

        for path in &paths {
            if path.exists() {
                let _ = watcher.watch(path, RecursiveMode::Recursive);
                self.watched_paths.insert(path.clone());
            }
        }

        self.watcher = Some(watcher);

        // Spawn debouncer task
        tokio::spawn(async move {
            let debounce_duration = Duration::from_millis(500);
            let mut last_event = None;

            loop {
                tokio::select! {
                    Some(_event) = rx.recv() => {
                        last_event = Some(tokio::time::Instant::now());
                    }
                    _ = sleep(Duration::from_millis(100)) => {
                        if let Some(instant) = last_event {
                            if instant.elapsed() >= debounce_duration {
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

        // Wait for debounce (500ms + buffer)
        sleep(Duration::from_millis(1000)).await;

        // Should have triggered only once due to debounce
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        manager.stop();
    }
}
