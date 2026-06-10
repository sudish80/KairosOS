use kairos_tui::config::Config;
use kairos_tui::telemetry::Telemetry;

#[test]
fn test_config_default_resolution() {
    let cfg = Config::default();
    assert!(cfg.width == 1920 || cfg.width == 0);
}

#[test]
fn test_telemetry_render_count() {
    let t = Telemetry::new();
    assert_eq!(t.render_count(), 0);
    t.incr_render_count();
    assert_eq!(t.render_count(), 1);
}

#[test]
fn test_config_fb_device_path() {
    let cfg = Config::default();
    assert!(cfg.fb_device == "/dev/fb0" || cfg.fb_device.is_empty());
}
