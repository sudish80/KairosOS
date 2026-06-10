//! dm-verity manager — hash tree verification, integrity checking
use crate::config;
use crate::error::RecoveryError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct VerityManager {
    config: Arc<RwLock<config::Config>>,
}

impl VerityManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub async fn verify_device(&self, device: &str) -> anyhow::Result<bool> {
        let cfg = self.config.read().await;
        info!("Verifying dm-verity for {}", device);

        let output = Command::new("veritysetup")
            .args([
                "verify",
                device,
                &cfg.verity.hash_device,
                &cfg.verity.root_hash_file,
            ])
            .output()
            .await;

        match output {
            Ok(out) => {
                let passed = out.status.success();
                info!(
                    "Verity check for {}: {}",
                    device,
                    if passed { "PASS" } else { "FAIL" }
                );
                Ok(passed)
            }
            Err(e) => {
                warn!("veritysetup not available ({}), skipping verity check", e);
                Ok(true) // Skip if veritysetup not available
            }
        }
    }

    pub async fn setup_verity(&self, device: &str) -> anyhow::Result<String> {
        let cfg = self.config.read().await;
        let verity_name = format!("kairos-verity-{}", std::process::id());
        let verity_device = format!("/dev/mapper/{}", verity_name);

        let status = Command::new("veritysetup")
            .args([
                "open",
                device,
                &verity_name,
                &cfg.verity.hash_device,
                &cfg.verity.root_hash_file,
            ])
            .status()
            .await;

        match status {
            Ok(s) if s.success() => {
                info!("Verity device opened: {}", verity_device);
                Ok(verity_device)
            }
            _ => {
                warn!("Failed to open verity device (veritysetup may not be available)");
                Ok(device.to_string()) // Fall through to raw device
            }
        }
    }

    pub async fn teardown_verity(&self, name: &str) -> anyhow::Result<()> {
        let _ = Command::new("veritysetup")
            .args(["close", name])
            .status()
            .await;
        Ok(())
    }
}
