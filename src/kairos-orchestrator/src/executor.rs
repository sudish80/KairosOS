use std::sync::Arc; use tokio::sync::RwLock; use tokio::process::Command;
use tracing::{info, error}; use crate::config; use crate::resource::ResourceManager;
pub struct TaskExecutor { config: Arc<RwLock<config::Config>>, resource_manager: Arc<ResourceManager> }
impl TaskExecutor {
    pub fn new(config: Arc<RwLock<config::Config>>, resource_manager: Arc<ResourceManager>) -> Self { Self { config, resource_manager } }
    pub async fn execute(&self, command: &str, timeout_secs: u64) -> anyhow::Result<(bool, String)> {
        let output = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), Command::new("sh").args(["-c", command]).output()).await;
        match output {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                info!("Command executed (success: {}): {}", out.status.success(), &command[..command.len().min(80)]);
                Ok((out.status.success(), stdout))
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Execution failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Command timed out after {}s: {}", timeout_secs, &command[..command.len().min(80)])),
        }
    }
}
