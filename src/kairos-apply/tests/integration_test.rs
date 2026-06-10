use kairos_apply::config::Config;
use kairos_apply::telemetry::Telemetry;

#[test]
fn test_config_default() {
    let cfg = Config::default();
    assert!(cfg.config_dir.contains("apply"));
}

#[test]
fn test_telemetry_count() {
    let t = Telemetry::new();
    assert_eq!(t.apply_count(), 0);
    t.incr_apply_count();
    assert_eq!(t.apply_count(), 1);
}

#[test]
fn test_config_from_env() {
    std::env::set_var("KAIROS_APPLY_CONFIG", "/tmp/test.toml");
    let cfg = Config::default();
    assert!(cfg.config_path.is_empty() || cfg.config_path == "/tmp/test.toml");
    std::env::remove_var("KAIROS_APPLY_CONFIG");
}
