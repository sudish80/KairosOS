use std::sync::Arc; use tokio::sync::RwLock; use tokio::process::Command;
use tracing::info; use crate::config;
pub struct Quantizer { config: Arc<RwLock<config::Config>> }
impl Quantizer {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn quantize(&self, input: &str, output: &str, quant_type: &str) -> anyhow::Result<()> {
        let status = Command::new("llama-quantize")
            .args([input, output, quant_type]).status().await
            .map_err(|e| anyhow::anyhow!("llama-quantize not available: {}", e))?;
        if status.success() { info!("Quantized {} -> {} ({})", input, output, quant_type); Ok(()) }
        else { Err(anyhow::anyhow!("Quantization failed for {}", input)) }
    }
}
