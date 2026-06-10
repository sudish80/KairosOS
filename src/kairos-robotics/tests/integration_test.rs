use kairos_robotics::config::Config;
use kairos_robotics::telemetry::Telemetry;

#[test]
fn test_config_default_pwm_device() {
    let cfg = Config::default();
    assert!(cfg.pwm_device == "/dev/pwm" || cfg.pwm_device.is_empty());
}

#[test]
fn test_telemetry_control_loop_count() {
    let t = Telemetry::new();
    assert_eq!(t.control_loop_count(), 0);
    t.incr_control_loop_count();
    assert_eq!(t.control_loop_count(), 1);
}

#[test]
fn test_config_dh_params_path() {
    let cfg = Config::default();
    assert!(cfg.dh_params_path.contains("dh.toml") || cfg.dh_params_path.is_empty());
}
