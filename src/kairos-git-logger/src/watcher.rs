//! File system watcher with debounce, ignore patterns, and change coalescing
//! Uses inotify on Linux via notify crate, falls back to polling
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use notify::{Config as NotifyConfig, Event, EventKind, RecursiveMode, Watcher as NotifyWatcher};
use tracing::{info, debug, error, warn};
use crate::config;
use crate::committer::ChangeCommitter;
use crate::telemetry::Telemetry;

static IGNORE_PATTERNS: &[&str] = &[
    ".swp", ".tmp", ".lock", "~", ".git", ".git/index",
    "*.pid", "*.sock", "gen.json",
];

pub struct FileWatcher {
    config: Arc<RwLock<config::Config>>,
    committer: Arc<ChangeCommitter>,
    telemetry: Arc<Telemetry>,
    debounced_changes: Arc<RwLock<HashMap<PathBuf, Instant>>>,
}

impl FileWatcher {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        committer: Arc<ChangeCommitter>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        Self { config, committer, telemetry, debounced_changes: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn should_ignore(path: &std::path::Path) -> bool {
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let path_str = path.to_string_lossy();
        for pat in IGNORE_PATTERNS {
            if pat.starts_with('*') {
                if name.ends_with(&pat[1..]) {
                    return true;
                }
            } else if path_str.contains(pat) {
                return true;
            }
        }
        false
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let cfg = self.config.read().await;
        let watch_paths: Vec<PathBuf> = cfg.watcher.watch_paths.iter().map(PathBuf::from).collect();
        let debounce_ms = cfg.watcher.debounce_ms;
        drop(cfg);

        info!("FileWatcher starting, watching {:?}", watch_paths);

        let debounced = Arc::clone(&self.debounced_changes);
        let committer = Arc::clone(&self.committer);
        let telemetry = Arc::clone(&self.telemetry);

        // Build the notify watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Filter to modify/create events only
                let is_relevant = matches!(event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                );
                if !is_relevant {
                    return;
                }

                for path in &event.paths {
                    if Self::should_ignore(path) {
                        continue;
                    }
                    telemetry.record_watcher_event();
                    let mut changes = debounced.blocking_write();
                    changes.insert(path.clone(), Instant::now());
                }
            }
        }).map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;

        for path in &watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)
                    .map_err(|e| anyhow::anyhow!("Failed to watch {}: {}", path.display(), e))?;
                info!("Watching: {}", path.display());
            } else {
                warn!("Watch path does not exist, skipping: {}", path.display());
            }
        }

        // Debounce loop — collects changes and commits after quiet period
        let debounced2 = Arc::clone(&self.debounced_changes);
        let committer2 = Arc::clone(&committer);
        let telemetry2 = Arc::clone(&telemetry);
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(debounce_ms));
            loop {
                interval.tick().await;
                let ready: Vec<PathBuf> = {
                    let mut changes = debounced2.write().await;
                    let now = Instant::now();
                    let mut ready = Vec::new();
                    changes.retain(|path, time| {
                        if now.duration_since(*time) >= Duration::from_millis(debounce_ms) {
                            ready.push(path.clone());
                            false
                        } else {
                            true
                        }
                    });
                    ready
                };

                if !ready.is_empty() {
                    debug!("Debounced {} changes, committing...", ready.len());
                    match committer2.commit_changes(&ready).await {
                        Ok(hash) => {
                            if !hash.is_empty() {
                                info!("Committed {} changed files as {}", ready.len(), &hash[..hash.len().min(12)]);
                            }
                        }
                        Err(e) => {
                            error!("Commit failed for {} files: {}", ready.len(), e);
                            telemetry2.record_error();
                        }
                    }
                }
            }
        });

        // Keep watcher alive until shutdown
        let _watcher = watcher;
        handle.await?;
        Ok(())
    }
}
