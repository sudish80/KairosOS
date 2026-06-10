// Generation management — create, list, activate config generations

use crate::parser::{ParsedConfig, format_config};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::fs;

const GENERATIONS_DIR: &str = "/etc/kairos/generations";
const ACTIVE_LINK: &str = "/etc/kairos/generations/active";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    pub id: String,
    pub created: DateTime<Utc>,
    pub active: bool,
    pub config_hash: String,
    pub description: Option<String>,
}

pub fn create(config: &ParsedConfig) -> Result<String> {
    let now = Utc::now();
    let config_json = format_config(config, "json")?;
    let hash = hex::encode(Sha256::digest(config_json.as_bytes()));

    let ts = now.format("%Y%m%d-%H%M%S").to_string();
    let id = format!("gen-{}-{}", ts, &hash[..8]);
    let gen_dir = PathBuf::from(GENERATIONS_DIR).join(&id);

    fs::create_dir_all(&gen_dir)
        .context("Failed to create generation directory")?;

    let generation = Generation {
        id: id.clone(),
        created: now,
        active: false,
        config_hash: hash,
        description: None,
    };

    // Save the configuration
    fs::write(gen_dir.join("config.json"), &config_json)?;
    fs::write(gen_dir.join("config.yaml"), format_config(config, "yaml")?)?;

    // Save the generation metadata
    let meta = serde_json::to_string_pretty(&generation)?;
    fs::write(gen_dir.join("generation.json"), &meta)?;

    Ok(id)
}

pub fn preview(config: &ParsedConfig) -> Result<()> {
    println!("Would create generation with:");
    println!("  Agent:     hermes={}, openclaw={}",
        config.agent.hermes.is_some(),
        config.agent.openclaw.is_some());
    println!("  Services:  sshd={}, docker={}",
        config.services.sshd.is_some(),
        config.services.docker.is_some());
    println!("  AI:        ollama={}, knowledge_graph={}",
        config.ai.ollama.is_some(),
        config.ai.knowledge_graph.is_some());
    Ok(())
}

pub fn activate(id: &str) -> Result<()> {
    let gen_path = PathBuf::from(GENERATIONS_DIR).join(id);
    if !gen_path.exists() {
        return Err(anyhow::anyhow!("Generation {} not found", id));
    }

    // Update active symlink
    let _ = fs::remove_file(ACTIVE_LINK);
    std::os::unix::fs::symlink(&gen_path, ACTIVE_LINK)
        .context("Failed to create active symlink")?;

    // Update generation metadata
    let meta_path = gen_path.join("generation.json");
    if let Ok(content) = fs::read_to_string(&meta_path) {
        if let Ok(mut gen) = serde_json::from_str::<Generation>(&content) {
            gen.active = true;
            fs::write(&meta_path, serde_json::to_string_pretty(&gen)?)?;
        }
    }

    Ok(())
}

pub fn list() -> Result<Vec<Generation>> {
    let dir = PathBuf::from(GENERATIONS_DIR);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut generations = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let meta_path = entry.path().join("generation.json");
        if meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(gen) = serde_json::from_str::<Generation>(&content) {
                    generations.push(gen);
                }
            }
        }
    }

    generations.sort_by(|a, b| b.created.cmp(&a.created));
    Ok(generations)
}

pub fn current_config() -> Result<Option<serde_json::Value>> {
    let active_path = PathBuf::from(ACTIVE_LINK);
    if !active_path.exists() {
        return Ok(None);
    }

    let config_path = active_path.join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_path)?;
    Ok(Some(serde_json::from_str(&content)?))
}
