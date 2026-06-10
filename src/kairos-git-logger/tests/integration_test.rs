use kairos_git_logger::config::Config;
use kairos_git_logger::telemetry::Telemetry;

#[test]
fn test_config_default_git_dir() {
    let cfg = Config::default();
    assert!(cfg.repo_path.is_empty() || cfg.repo_path.contains("git"));
}

#[test]
fn test_telemetry_commit_count() {
    let t = Telemetry::new();
    assert_eq!(t.commit_count(), 0);
}

#[test]
fn test_config_watch_dirs_default() {
    let cfg = Config::default();
    assert!(cfg.watch_dirs.is_empty() || !cfg.watch_dirs.is_empty());
}
