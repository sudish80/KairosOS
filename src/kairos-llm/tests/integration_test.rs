use kairos_llm::config::Config;
use kairos_llm::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_model_dir() {
    let cfg = Config::default();
    assert!(cfg.models.models_dir.contains("models") || cfg.models.models_dir.is_empty());
}

#[test]
fn test_telemetry_record_inference() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_inference(512);
    let m = t.metrics();
    assert_eq!(m["inferences"], 1);
    assert_eq!(m["tokens_generated"], 512);
}

#[test]
fn test_config_context_size_default() {
    let cfg = Config::default();
    assert!(cfg.runtime.ctx_size == 4096 || cfg.runtime.ctx_size == 0);
}
