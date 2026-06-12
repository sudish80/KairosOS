use kairos_robotics::config::Config;
use kairos_robotics::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_pwm_device() {
    let cfg = Config::default();
    assert!(cfg.control.pwm_device == "/dev/pwm" || cfg.control.pwm_device.is_empty());
}

#[test]
fn test_telemetry_record_loop() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_loop();
    let m = t.metrics();
    assert_eq!(m["control_loops"], 1);
}

#[test]
fn test_config_dh_params() {
    let cfg = Config::default();
    assert!(cfg.kinematics.dh_params.contains("dh.toml") || cfg.kinematics.dh_params.is_empty());
}
