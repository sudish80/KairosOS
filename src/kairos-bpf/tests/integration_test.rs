use kairos_bpf::config::Config;
use kairos_bpf::telemetry::TelemetryStore;

#[test]
fn test_config_default_endpoint() {
    let cfg = Config::default();
    assert_eq!(cfg.endpoints.get("bpf").unwrap(), "unix:///var/run/kairos/bpf.sock");
}

#[test]
fn test_telemetry_store_healthy() {
    let t = TelemetryStore::new().unwrap();
    assert!(t.is_healthy());
}

#[test]
fn test_config_from_file_nonexistent() {
    let cfg = Config::from_file("/nonexistent/bpf.toml");
    assert!(cfg.is_err());
}
