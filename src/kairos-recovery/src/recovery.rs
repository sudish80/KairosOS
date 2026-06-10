//! Recovery shell — emergency access with controlled environment
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command;
use tracing::{info, warn};
use crate::config;

pub struct RecoveryShell {
    config: Arc<RwLock<config::Config>>,
}

impl RecoveryShell {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub async fn enter_shell(&self, reason: &str) -> anyhow::Result<()> {
        let cfg = self.config.read().await;
        if !cfg.recovery.shell_enabled {
            warn!("Recovery shell is disabled");
            return Ok(());
        }

        info!("Entering recovery shell: {}", reason);

        // In production: launch restricted shell in recovery chroot
        let shell_path = &cfg.recovery.shell_path;
        println!("\n=== KAIROS RECOVERY SHELL ===");
        println!("Reason: {}", reason);
        println!("Type 'exit' to return to normal boot\n");

        let status = Command::new(shell_path).status().await?;
        info!("Recovery shell exited with: {:?}", status.code());
        Ok(())
    }

    pub async fn run_recovery_command(&self, cmd: &str) -> anyhow::Result<String> {
        let output = Command::new("sh")
            .args(["-c", cmd])
            .output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
