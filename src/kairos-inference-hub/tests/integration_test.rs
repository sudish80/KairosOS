use kairos_inference_hub::config::Config;
use kairos_inference_hub::telemetry::Telemetry;

#[test]
fn test_config_default_model_paths() {
    let cfg = Config::default();
    assert!(cfg.draft_model.contains("draft") || cfg.draft_model.is_empty());
}

#[test]
fn test_telemetry_predictions() {
    let t = Telemetry::new();
    assert_eq!(t.accepted_tokens(), 0);
    t.incr_accepted(100);
    assert_eq!(t.accepted_tokens(), 100);
}

#[test]
fn test_telemetry_draft_ratio_on_init() {
    let t = Telemetry::new();
    assert_eq!(t.draft_count(), 0);
    assert_eq!(t.accepted_tokens(), 0);
}
