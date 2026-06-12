use kairos_inference_hub::config::Config;
use kairos_inference_hub::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_model_paths() {
    let cfg = Config::default();
    assert!(cfg.models.draft_model.contains("draft") || cfg.models.draft_model.is_empty());
}

#[test]
fn test_telemetry_record_request() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_request(500, 400, 380, 1_000_000);
    let m = t.metrics();
    assert_eq!(m["requests"], 1);
    assert_eq!(m["total_tokens"], 500);
    assert_eq!(m["draft_tokens"], 400);
    assert_eq!(m["accepted_tokens"], 380);
}

#[test]
fn test_telemetry_initial_cache_counts() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    let m = t.metrics();
    assert_eq!(m["cache_hits"], 0);
    assert_eq!(m["cache_misses"], 0);
}
