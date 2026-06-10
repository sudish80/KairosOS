use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub store: StoreConfig,
    pub validation: ValidationConfig,
    pub rollback: RollbackConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub state_dir: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub active_link: String,
    pub pending_dir: String,
    pub history_dir: String,
    pub max_generations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub schema_dir: String,
    pub strict: bool,
    pub verify_checksums: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    pub enabled: bool,
    pub auto_rollback: bool,
    pub max_attempts: u32,
    pub health_check_timeout_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                state_dir: "/var/lib/kairos/apply".into(),
                log_level: "info".into(),
            },
            store: StoreConfig {
                active_link: "/etc/kairos/active".into(),
                pending_dir: "/var/lib/kairos/apply/pending".into(),
                history_dir: "/var/lib/kairos/apply/history".into(),
                max_generations: 10,
            },
            validation: ValidationConfig {
                schema_dir: "/etc/kairos/schemas".into(),
                strict: true,
                verify_checksums: true,
            },
            rollback: RollbackConfig {
                enabled: true,
                auto_rollback: true,
                max_attempts: 3,
                health_check_timeout_secs: 30,
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
