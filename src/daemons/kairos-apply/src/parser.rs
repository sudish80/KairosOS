// Configuration parser — supports YAML, TOML, and JSON formats
// Returns a unified parsed representation

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ConfigMap = HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedConfig {
    pub format: String,
    pub raw: ConfigMap,
    pub agent: AgentConfig,
    pub services: ServicesConfig,
    pub system: SystemConfig,
    pub ai: AiConfig,
    pub users: UsersConfig,
    pub hardware: HardwareConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub hermes: Option<serde_json::Value>,
    pub openclaw: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServicesConfig {
    pub sshd: Option<serde_json::Value>,
    pub ntp: Option<serde_json::Value>,
    pub firewall: Option<serde_json::Value>,
    pub docker: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemConfig {
    pub hostname: Option<String>,
    pub kernel: Option<serde_json::Value>,
    pub power: Option<serde_json::Value>,
    pub updates: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    pub ollama: Option<serde_json::Value>,
    pub knowledge_graph: Option<serde_json::Value>,
    pub ebpf_telemetry: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsersConfig {
    pub kairos: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareConfig {
    pub gpu: Option<serde_json::Value>,
    pub tpm: Option<serde_json::Value>,
    pub usbguard: Option<serde_json::Value>,
}

pub fn parse_yaml_config(content: &str) -> Result<ParsedConfig> {
    let raw: ConfigMap = serde_yaml::from_str(content)
        .context("Failed to parse YAML configuration")?;

    Ok(ParsedConfig {
        format: "yaml".into(),
        agent: serde_json::from_value(
            raw.get("agent").cloned().unwrap_or(serde_json::json!({}))
        )?,
        services: serde_json::from_value(
            raw.get("services").cloned().unwrap_or(serde_json::json!({}))
        )?,
        system: serde_json::from_value(
            raw.get("system").cloned().unwrap_or(serde_json::json!({}))
        )?,
        ai: serde_json::from_value(
            raw.get("ai").cloned().unwrap_or(serde_json::json!({}))
        )?,
        users: serde_json::from_value(
            raw.get("users").cloned().unwrap_or(serde_json::json!({}))
        )?,
        hardware: serde_json::from_value(
            raw.get("hardware").cloned().unwrap_or(serde_json::json!({}))
        )?,
        raw,
    })
}

pub fn parse_toml_config(content: &str) -> Result<ParsedConfig> {
    let raw: ConfigMap = toml::from_str(content)
        .context("Failed to parse TOML configuration")?;
    Ok(ParsedConfig {
        format: "toml".into(),
        agent: Default::default(),
        services: Default::default(),
        system: Default::default(),
        ai: Default::default(),
        users: Default::default(),
        hardware: Default::default(),
        raw,
    })
}

pub fn format_config(config: &ParsedConfig, format: &str) -> Result<String> {
    match format {
        "yaml" => serde_yaml::to_string(&config.raw)
            .context("Failed to serialize to YAML"),
        "json" => serde_json::to_string_pretty(&config.raw)
            .context("Failed to serialize to JSON"),
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
}
