use kairos_orchestrator::config::Config;
use kairos_orchestrator::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_timeout() {
    let cfg = Config::default();
    assert!(cfg.executor.timeout_secs > 0);
}

#[test]
fn test_telemetry_record_task() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_submission();
    t.record_completion();
    let m = t.metrics();
    assert_eq!(m["submitted"], 1);
    assert_eq!(m["completed"], 1);
}

#[test]
fn test_telemetry_initial_metrics() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    let m = t.metrics();
    assert_eq!(m["submitted"], 0);
    assert_eq!(m["completed"], 0);
}
