use std::sync::Arc; use tokio::sync::RwLock; use tokio::process::Command;
use tracing::{info, error}; use crate::config; use crate::model::ModelRegistry;
pub struct LlmRuntime { config: Arc<RwLock<config::Config>>, model_registry: Arc<ModelRegistry> }
impl LlmRuntime {
    pub fn new(config: Arc<RwLock<config::Config>>, model_registry: Arc<ModelRegistry>) -> Self { Self { config, model_registry } }
    pub async fn infer(&self, model: &str, prompt: &str) -> anyhow::Result<String> {
        let cfg = self.config.read().await;
        let model_path = format!("{}/{}", cfg.models.models_dir, model);
        if !std::path::Path::new(&model_path).exists() { return Err(anyhow::anyhow!("Model not found: {}", model_path)); }
        let output = Command::new(&cfg.runtime.llama_bin)
            .args(["-m", &model_path, "-p", prompt, "-n", "256", "--temp", "0.7", "--no-display-prompt"])
            .output().await.map_err(|e| anyhow::anyhow!("llama-server failed: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
