use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub models: ModelsConfig,
    pub kv_cache: KVCacheConfig,
    pub speculative: SpeculativeConfig,
    pub quantizer: QuantizerConfig,
    pub scheduler: SchedulerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
    pub max_concurrent: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    pub models_dir: String,
    pub draft_model: String,
    pub oracle_model: String,
    pub fallback_model: String,
    pub load_all_at_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVCacheConfig {
    pub max_entries: usize,
    pub entry_ttl_secs: u64,
    pub enable_compression: bool,
    pub page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeculativeConfig {
    pub enabled: bool,
    pub draft_length: usize,
    pub max_speculations: usize,
    pub acceptance_threshold: f64,
    pub fallback_on_reject: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizerConfig {
    pub default_precision: String,
    pub supported_precisions: Vec<String>,
    pub calibration_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub strategy: String,
    pub max_batch_size: usize,
    pub max_wait_ms: u64,
    pub priority_levels: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                log_level: "info".into(),
                max_concurrent: 4,
            },
            models: ModelsConfig {
                models_dir: "/var/lib/kairos/models".into(),
                draft_model: "draft-small".into(),
                oracle_model: "oracle-large".into(),
                fallback_model: "fallback-medium".into(),
                load_all_at_start: false,
            },
            kv_cache: KVCacheConfig {
                max_entries: 1024,
                entry_ttl_secs: 3600,
                enable_compression: true,
                page_size: 4096,
            },
            speculative: SpeculativeConfig {
                enabled: true,
                draft_length: 64,
                max_speculations: 5,
                acceptance_threshold: 0.9,
                fallback_on_reject: true,
            },
            quantizer: QuantizerConfig {
                default_precision: "fp16".into(),
                supported_precisions: vec![
                    "fp32".into(),
                    "fp16".into(),
                    "int8".into(),
                    "int4".into(),
                ],
                calibration_size: 100,
            },
            scheduler: SchedulerConfig {
                strategy: "fifo".into(),
                max_batch_size: 8,
                max_wait_ms: 100,
                priority_levels: 3,
            },
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
