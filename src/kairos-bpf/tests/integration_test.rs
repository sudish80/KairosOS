use kairos_bpf::config::Config;
use kairos_bpf::telemetry::Telemetry;

#[test]
fn test_config_default() {
    let cfg = Config::default();
    assert_eq!(cfg.socket_path, "/var/run/kairos/bpf.sock");
}

#[test]
fn test_telemetry_incr() {
    let t = Telemetry::new();
    let v = t.incr_eval_count();
    assert_eq!(v, 1);
    assert!(t.eval_count() >= 1);
}

#[test]
fn test_config_from_path_nonexistent() {
    let cfg = Config::from_path("/nonexistent/bpf.toml");
    assert!(cfg.is_err());
}

#[test]
fn test_thermal_zone_count_default() {
    let t = Telemetry::new();
    assert_eq!(t.thermal_events(), 0);
}
