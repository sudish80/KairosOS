use kairos_fb::config::Config;
use kairos_fb::telemetry::Telemetry;

#[test]
fn test_config_default_fb_device() {
    let cfg = Config::default();
    assert!(cfg.fb_device == "/dev/fb0" || cfg.fb_device.is_empty());
}

#[test]
fn test_telemetry_frame_count() {
    let t = Telemetry::new();
    assert_eq!(t.frame_count(), 0);
    t.incr_frame_count();
    assert_eq!(t.frame_count(), 1);
}

#[test]
fn test_config_resolution_defaults() {
    let cfg = Config::default();
    assert!(cfg.width == 1920 || cfg.width == 0);
}
