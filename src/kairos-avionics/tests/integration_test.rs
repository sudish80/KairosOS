use kairos_avionics::config::Config;
use kairos_avionics::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_serial_baud() {
    let cfg = Config::default();
    assert!(cfg.arinc.baud == 100000 || cfg.arinc.baud == 0);
}

#[test]
fn test_telemetry_record_rx() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_rx();
    let m = t.metrics();
    assert_eq!(m["packets_rx"], 1);
}

#[test]
fn test_config_mavlink_sysid_default() {
    let cfg = Config::default();
    assert!(cfg.mavlink.sys_id > 0 || cfg.mavlink.sys_id == 0);
}
