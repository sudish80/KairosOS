use kairos_fb::config::Config;
use kairos_fb::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_fb_device() {
    let cfg = Config::default();
    assert!(cfg.display.fb_device == "/dev/fb0" || cfg.display.fb_device.is_empty());
}

#[test]
fn test_telemetry_record_frame() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_frame();
    let m = t.metrics();
    assert_eq!(m["frames_rendered"], 1);
}

#[test]
fn test_config_resolution_defaults() {
    let cfg = Config::default();
    assert!(cfg.display.width == 1920 || cfg.display.width == 0);
}
