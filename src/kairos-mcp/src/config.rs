//! Configuration — TOML-based with environment override support
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub transport: TransportConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub audit: AuditConfig,
    pub plugin: PluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub directory: String,
    pub max_plugins: u32,
    pub sandbox: bool,
    pub memory_limit_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub pid_file: String,
    pub log_level: String,
    pub shutdown_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub unix_socket_path: String,
    pub tcp_bind: String,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub max_connections: u32,
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub token_ttl_secs: u64,
    pub tls_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub capacity: u32,
    pub refill_per_sec: f64,
    pub burst_multiplier: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_path: String,
    pub log_auth_events: bool,
    pub log_api_calls: bool,
    pub log_config_changes: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                pid_file: "/var/run/kairos/mcp.pid".into(),
                log_level: "info".into(),
                shutdown_timeout_secs: 10,
            },
            transport: TransportConfig {
                unix_socket_path: "/var/run/kairos/mcp.sock".into(),
                tcp_bind: "127.0.0.1:8080".into(),
                tls_cert: None,
                tls_key: None,
                max_connections: 1024,
                connection_timeout_secs: 30,
            },
            auth: AuthConfig {
                enabled: true,
                token_ttl_secs: 3600,
                tls_required: false,
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                capacity: 100,
                refill_per_sec: 10.0,
                burst_multiplier: 2,
            },
            audit: AuditConfig {
                enabled: true,
                log_path: "/var/log/kairos/audit.log".into(),
                log_auth_events: true,
                log_api_calls: true,
                log_config_changes: true,
            },
            plugin: PluginConfig {
                enabled: true,
                directory: "/usr/lib/kairos/plugins".into(),
                max_plugins: 50,
                sandbox: true,
                memory_limit_mb: 64,
            },
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = toml::from_str(&content)?;

        // Environment variable overrides
        if let Ok(val) = std::env::var("KAIROS_MCP_LOG_LEVEL") {
            cfg.general.log_level = val;
        }
        if let Ok(val) = std::env::var("KAIROS_MCP_BIND") {
            cfg.transport.tcp_bind = val;
        }
        if let Ok(val) = std::env::var("KAIROS_MCP_SOCKET") {
            cfg.transport.unix_socket_path = val;
        }
        if let Ok(val) = std::env::var("KAIROS_MCP_AUTH_ENABLED") {
            cfg.auth.enabled = val == "true" || val == "1";
        }

        Ok(cfg)
    }
}
