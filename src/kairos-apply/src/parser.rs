//! Declarative config parser (YAML/HCL/TOML/JSON)
use crate::config;
use crate::error::ApplyError;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeclarativeConfig {
    pub version: String,
    pub metadata: ConfigMetadata,
    pub files: HashMap<String, FileSpec>,
    pub system: SystemConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigMetadata {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSpec {
    pub path: String,
    pub content: Option<String>,
    pub source: Option<String>,
    pub mode: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub template: Option<bool>,
    pub validate: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemConfig {
    pub hostname: Option<String>,
    pub services: HashMap<String, ServiceSpec>,
    pub sysctls: HashMap<String, String>,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceSpec {
    pub enabled: bool,
    pub state: String,
    pub args: Vec<String>,
}

pub struct DeclarativeParser {
    config: Arc<RwLock<config::Config>>,
}

impl DeclarativeParser {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub fn parse(&self, path: &Path) -> anyhow::Result<DeclarativeConfig> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let content = std::fs::read_to_string(path)?;
        match ext {
            "yaml" | "yml" => Ok(serde_yaml::from_str(&content)?),
            "toml" => Ok(toml::from_str(&content)?),
            "json" => Ok(serde_json::from_str(&content)?),
            "hcl" => Err(ApplyError::Parse("HCL parser not yet implemented".into()).into()),
            _ => Err(ApplyError::Parse(format!("Unsupported format: {}", ext)).into()),
        }
    }

    pub fn to_file_specs(&self, config: &DeclarativeConfig) -> Vec<(String, Vec<u8>)> {
        config
            .files
            .iter()
            .map(|(name, spec)| {
                let content = spec.content.clone().unwrap_or_default();
                (spec.path.clone(), content.into_bytes())
            })
            .collect()
    }
}
