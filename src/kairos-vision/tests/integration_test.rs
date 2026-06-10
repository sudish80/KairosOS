use kairos_vision::config::Config;
use kairos_vision::telemetry::Telemetry;

#[test]
fn test_config_default_video_device() {
    let cfg = Config::default();
    assert!(cfg.video_device == "/dev/video0" || cfg.video_device.is_empty());
}

#[test]
fn test_telemetry_frame_count() {
    let t = Telemetry::new();
    assert_eq!(t.frame_count(), 0);
    t.incr_frame_count();
    assert_eq!(t.frame_count(), 1);
}

#[test]
fn test_config_yolo_model_path() {
    let cfg = Config::default();
    assert!(cfg.model_path.contains(".gguf") || cfg.model_path.is_empty());
}
