use kairos_mcp::config::Config;
use kairos_mcp::telemetry::Telemetry;

#[test]
fn test_config_default_socket() {
    let cfg = Config::default();
    assert_eq!(cfg.socket_path, "/var/run/kairos/mcp.sock");
}

#[test]
fn test_service_registry_empty_on_init() {
    let t = Telemetry::new();
    assert_eq!(t.eval_count(), 0);
}

#[test]
fn test_telemetry_request_count() {
    let t = Telemetry::new();
    let c = t.incr_eval_count();
    assert_eq!(c, 1);
}

#[test]
fn test_config_deserialize_defaults() {
    let cfg = Config::default();
    assert!(cfg.log_level.is_empty() || cfg.log_level == "info");
}
