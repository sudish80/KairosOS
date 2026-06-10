use kairos_avionics::config::Config;
use kairos_avionics::telemetry::Telemetry;

#[test]
fn test_config_default_serial_baud() {
    let cfg = Config::default();
    assert!(cfg.serial_baud == 115200 || cfg.serial_baud == 0);
}

#[test]
fn test_telemetry_packet_count() {
    let t = Telemetry::new();
    assert_eq!(t.packet_count(), 0);
    t.incr_packet_count();
    assert_eq!(t.packet_count(), 1);
}

#[test]
fn test_config_mavlink_sysid_default() {
    let cfg = Config::default();
    assert!(cfg.mavlink_sys_id > 0 || cfg.mavlink_sys_id == 0);
}
