use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub dag: DagConfig,
    pub scheduler: SchedulerConfig,
    pub executor: ExecutorConfig,
    pub resource: ResourceConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
    pub max_concurrent_pipelines: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagConfig {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub allow_cycles: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub strategy: String,
    pub max_retries: u32,
    pub retry_delay_secs: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    pub max_parallel: usize,
    pub timeout_secs: u64,
    pub workspace: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_limit: f64,
    pub memory_limit_mb: u64,
    pub disk_limit_gb: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                log_level: "info".into(),
                max_concurrent_pipelines: 4,
            },
            dag: DagConfig {
                max_nodes: 1000,
                max_depth: 50,
                allow_cycles: false,
            },
            scheduler: SchedulerConfig {
                strategy: "critical-path".into(),
                max_retries: 3,
                retry_delay_secs: 10,
            },
            executor: ExecutorConfig {
                max_parallel: 8,
                timeout_secs: 3600,
                workspace: "/var/lib/kairos/orchestrator/workspace".into(),
            },
            resource: ResourceConfig {
                cpu_limit: 0.9,
                memory_limit_mb: 8192,
                disk_limit_gb: 50,
            },
        }
    }
}
impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }
}
