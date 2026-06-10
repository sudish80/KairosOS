use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub storage: StorageConfig, pub vector: VectorConfig, pub mem_bus: MemBusConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig { pub db_path: String, pub max_size_mb: u64, pub wal_enabled: bool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig { pub dimension: usize, pub index_type: String, pub max_vectors: usize }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemBusConfig { pub capacity: usize, pub channels: Vec<String>, pub ttl_secs: u64 }
impl Default for Config {
    fn default() -> Self { Self {
        general: GeneralConfig { daemonize: true, log_level: "info".into() },
        storage: StorageConfig { db_path: "/var/lib/kairos/db/kairos.db".into(), max_size_mb: 1024, wal_enabled: true },
        vector: VectorConfig { dimension: 768, index_type: "hnsw".into(), max_vectors: 1000000 },
        mem_bus: MemBusConfig { capacity: 10000, channels: vec!["events".into(), "metrics".into(), "alerts".into()], ttl_secs: 3600 },
    } }
}
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
