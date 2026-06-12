use kairos_bio::config::Config;
use kairos_bio::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_ref_genome() {
    let cfg = Config::default();
    assert!(cfg.sequence.reference_genome.contains("hg38") || cfg.sequence.reference_genome.is_empty());
}

#[test]
fn test_telemetry_record_sequence() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_sequence(1000);
    let m = t.metrics();
    assert_eq!(m["bases_aligned"], 1000);
}

#[test]
fn test_config_data_dir_default() {
    let cfg = Config::default();
    assert!(cfg.general.data_dir.contains("bio") || cfg.general.data_dir.is_empty());
}
