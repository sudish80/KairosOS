use kairos_tui::config::Config;
use kairos_tui::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_resolution() {
    let cfg = Config::default();
    assert!(cfg.display.width == 1920 || cfg.display.width == 0);
}

#[test]
fn test_telemetry_record_frame() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_frame(16_000_000);
    let m = t.metrics();
    assert_eq!(m["frames_rendered"], 1);
}

#[test]
fn test_config_fb_device_path() {
    let cfg = Config::default();
    assert!(cfg.display.framebuffer_device == "/dev/fb0" || cfg.display.framebuffer_device.is_empty());
}
