use kairos_orchestrator::config::Config;
use kairos_orchestrator::telemetry::Telemetry;

#[test]
fn test_config_default_timeout() {
    let cfg = Config::default();
    assert!(cfg.task_timeout_secs > 0);
}

#[test]
fn test_telemetry_task_count() {
    let t = Telemetry::new();
    assert_eq!(t.task_count(), 0);
    t.incr_task_count();
    assert_eq!(t.task_count(), 1);
}

#[test]
fn test_telemetry_success_rate_on_init() {
    let t = Telemetry::new();
    assert_eq!(t.success_count(), 0);
    assert_eq!(t.failure_count(), 0);
}
