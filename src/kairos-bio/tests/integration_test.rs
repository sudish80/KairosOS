use kairos_bio::config::Config;
use kairos_bio::telemetry::Telemetry;

#[test]
fn test_config_default_ref_genome() {
    let cfg = Config::default();
    assert!(cfg.ref_genome.contains("hg38") || cfg.ref_genome.is_empty());
}

#[test]
fn test_telemetry_sequence_count() {
    let t = Telemetry::new();
    assert_eq!(t.sequence_count(), 0);
    t.incr_sequence_count();
    assert_eq!(t.sequence_count(), 1);
}

#[test]
fn test_config_output_dir_default() {
    let cfg = Config::default();
    assert!(cfg.output_dir.contains("bio") || cfg.output_dir.is_empty());
}
