use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub repo: RepoConfig,
    pub watcher: WatcherConfig,
    pub commit: CommitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub bare_path: String,
    pub workdir: String,
    pub auto_init: bool,
    pub gc_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub watch_paths: Vec<String>,
    pub debounce_ms: u64,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitConfig {
    pub author_name: String,
    pub author_email: String,
    pub max_commit_interval_secs: u64,
    pub commit_message_prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig { daemonize: true, log_level: "info".into() },
            repo: RepoConfig {
                bare_path: "/var/lib/kairos/git/state.git".into(),
                workdir: "/etc".into(),
                auto_init: true,
                gc_interval_secs: 86400,
            },
            watcher: WatcherConfig {
                watch_paths: vec!["/etc".into(), "/var/lib/kairos".into()],
                debounce_ms: 1000,
                ignore_patterns: vec!["*.swp".into(), "*.tmp".into(), ".git".into(), "*.lock".into()],
            },
            commit: CommitConfig {
                author_name: "Kairos Git Logger".into(),
                author_email: "git-logger@kairos.local".into(),
                max_commit_interval_secs: 300,
                commit_message_prefix: "[kairos-git]".into(),
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
