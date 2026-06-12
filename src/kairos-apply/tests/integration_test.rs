use kairos_apply::config::Config;
use kairos_apply::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_state_dir() {
    let cfg = Config::default();
    assert!(cfg.general.state_dir.contains("apply"));
}

#[test]
fn test_telemetry_record_generation() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_generation_created();
    let m = t.metrics();
    assert_eq!(m["generations_created"], 1);
}

#[test]
fn test_config_default_values() {
    let cfg = Config::default();
    assert_eq!(cfg.general.daemonize, true);
    assert_eq!(cfg.store.max_generations, 10);
    assert_eq!(cfg.validation.strict, true);
    assert_eq!(cfg.rollback.enabled, true);
}
