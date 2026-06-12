use kairos_git_logger::config::Config;
use kairos_git_logger::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_git_dir() {
    let cfg = Config::default();
    assert!(cfg.repo.bare_path.contains("git") || cfg.repo.bare_path.is_empty());
}

#[test]
fn test_telemetry_record_commit() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_commit(5);
    let m = t.metrics();
    assert_eq!(m["commits"], 1);
    assert_eq!(m["files_changed"], 5);
}

#[test]
fn test_config_watch_dirs_default() {
    let cfg = Config::default();
    assert!(!cfg.watcher.watch_paths.is_empty());
}
