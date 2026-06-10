use kairos_llm::config::Config;
use kairos_llm::telemetry::Telemetry;

#[test]
fn test_config_default_model_path() {
    let cfg = Config::default();
    assert!(cfg.model_path.contains("models") || cfg.model_path.is_empty());
}

#[test]
fn test_telemetry_inference_count() {
    let t = Telemetry::new();
    assert_eq!(t.inference_count(), 0);
    t.incr_inference_count();
    assert_eq!(t.inference_count(), 1);
}

#[test]
fn test_config_context_size_default() {
    let cfg = Config::default();
    assert!(cfg.context_size == 4096 || cfg.context_size == 0);
}
