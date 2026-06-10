use kairos_recovery::config::Config;
use kairos_recovery::telemetry::Telemetry;
use kairos_recovery::update::{UpdateManifest, ImageEntry, DeltaEntry, UpdateCheckResult};
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_partition_labels() {
    let cfg = Config::default();
    assert!(cfg.partitions.slot_a.contains("KAIROS_A"));
    assert!(cfg.partitions.slot_b.contains("KAIROS_B"));
}

#[test]
fn test_config_ota_defaults() {
    let cfg = Config::default();
    assert!(cfg.update.server_url.contains("updates.kairosos.org"));
    assert_eq!(cfg.update.channel, "stable");
    assert_eq!(cfg.update.staging_percentage, 10);
    assert_eq!(cfg.update.auto_check, true);
    assert_eq!(cfg.update.auto_apply, false);
}

#[test]
fn test_telemetry_defaults() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    assert_eq!(t.updates_applied(), 0);
    assert_eq!(t.rollbacks_performed(), 0);
    assert_eq!(t.verity_checks(), 0);
}

#[test]
fn test_telemetry_record_update() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_update();
    t.record_update();
    assert_eq!(t.updates_applied(), 2);
}

#[test]
fn test_telemetry_record_verity() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_verity_check(true);
    t.record_verity_check(false);
    assert_eq!(t.verity_checks(), 2);
    assert_eq!(t.verity_failures(), 1);
}

#[test]
fn test_manifest_serialization() {
    let manifest = UpdateManifest {
        format_version: 1,
        release_id: "test-001".into(),
        version: "1.0.1".into(),
        channel: "stable".into(),
        staging_percentage: 10,
        images: vec![ImageEntry {
            target: "full".into(),
            url: "https://example.com/img.img".into(),
            sha256: "abcdef".into(),
            size_bytes: 1024,
            compression: "none".into(),
        }],
        deltas: vec![],
        min_bootloader_version: "1.0".into(),
        signing_key_fingerprint: "AABB".into(),
        timestamp: "2026-01-01T00:00:00Z".into(),
        description: "Test manifest".into(),
    };

    let json = serde_json::to_string_pretty(&manifest).unwrap();
    assert!(json.contains("test-001"));
    assert!(json.contains("1.0.1"));

    // Round-trip
    let deserialized: UpdateManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.release_id, "test-001");
    assert_eq!(deserialized.images.len(), 1);
    assert_eq!(deserialized.images[0].sha256, "abcdef");
}

#[test]
fn test_check_result_serialization() {
    let result = UpdateCheckResult {
        update_available: false,
        manifest: None,
        eligible: false,
        reason: "no update".into(),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("no update"));
}

#[test]
fn test_config_load_nonexistent_path() {
    let result = Config::load(std::path::Path::new("/nonexistent/kairos.toml"));
    assert!(result.is_err());
}

#[test]
fn test_config_update_download_dir() {
    let cfg = Config::default();
    assert!(cfg.update.download_dir.contains("updates"));
}

#[test]
fn test_delta_entry_roundtrip() {
    let delta = DeltaEntry {
        from_version: "1.0.0".into(),
        to_version: "1.0.1".into(),
        url: "https://example.com/delta.patch".into(),
        sha256: "123456".into(),
        size_bytes: 50000,
    };
    let json = serde_json::to_string(&delta).unwrap();
    let back: DeltaEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from_version, "1.0.0");
    assert_eq!(back.to_version, "1.0.1");
}

#[test]
fn test_image_entry_roundtrip() {
    let img = ImageEntry {
        target: "full".into(),
        url: "https://example.com/img.gz".into(),
        sha256: "deadbeef".into(),
        size_bytes: 2147483648,
        compression: "gzip".into(),
    };
    let json = serde_json::to_string(&img).unwrap();
    let back: ImageEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.compression, "gzip");
    assert_eq!(back.size_bytes, 2147483648);
}
