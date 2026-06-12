use kairos_vision::config::Config;
use kairos_vision::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_video_device() {
    let cfg = Config::default();
    assert!(cfg.capture.device == "/dev/video0" || cfg.capture.device.is_empty());
}

#[test]
fn test_telemetry_record_frame() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_frame();
    let m = t.metrics();
    assert_eq!(m["frames_processed"], 1);
}

#[test]
fn test_config_detect_model_path() {
    let cfg = Config::default();
    assert!(cfg.detect.model.contains(".gguf") || cfg.detect.model.is_empty());
}
