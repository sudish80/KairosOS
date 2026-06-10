//! Model registry — load, unload, version models dynamically
use crate::config;
use crate::error::InferenceError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct ModelHandle {
    pub name: String,
    pub path: PathBuf,
    pub precision: String,
    pub model_type: ModelType,
    pub loaded: bool,
    pub context_length: usize,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelType {
    Draft,
    Oracle,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct DraftOutput {
    pub tokens: Vec<String>,
    pub logits: Vec<f32>,
    pub confidences: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct OracleOutput {
    pub text: String,
    pub model_name: String,
    pub tokens_generated: usize,
}

pub struct ModelRegistry {
    config: Arc<RwLock<config::Config>>,
    models: Arc<RwLock<HashMap<String, ModelHandle>>>,
    models_dir: PathBuf,
}

impl ModelRegistry {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        let cfg = config.read().await;
        let models_dir = PathBuf::from(&cfg.models.models_dir);
        fs::create_dir_all(&models_dir).await?;

        Ok(Self {
            config,
            models: Arc::new(RwLock::new(HashMap::new())),
            models_dir,
        })
    }

    pub async fn load_model(&self, name: &str) -> anyhow::Result<ModelHandle> {
        let cfg = self.config.read().await;
        let model_path = self.models_dir.join(name);
        if !model_path.exists() {
            return Err(InferenceError::ModelNotFound(name.to_string()).into());
        }

        let model_type = if name == cfg.models.draft_model {
            ModelType::Draft
        } else if name == cfg.models.oracle_model {
            ModelType::Oracle
        } else {
            ModelType::Fallback
        };

        let handle = ModelHandle {
            name: name.to_string(),
            path: model_path,
            precision: cfg.quantizer.default_precision.clone(),
            model_type,
            loaded: true,
            context_length: 4096,
            metadata: HashMap::new(),
        };

        self.models
            .write()
            .await
            .insert(name.to_string(), handle.clone());
        info!("Loaded model: {} ({:?})", name, model_type);
        Ok(handle)
    }

    pub async fn get(&self, name: &str) -> Option<ModelHandle> {
        self.models.read().await.get(name).cloned()
    }

    pub async fn get_or_load(&self, name: &str) -> anyhow::Result<ModelHandle> {
        if let Some(model) = self.get(name).await {
            return Ok(model);
        }
        self.load_model(name).await
    }

    pub async fn unload(&self, name: &str) {
        self.models.write().await.remove(name);
        info!("Unloaded model: {}", name);
    }

    pub async fn list(&self) -> Vec<ModelHandle> {
        self.models.read().await.values().cloned().collect()
    }
}
