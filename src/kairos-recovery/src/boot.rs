//! Boot manager — slot selection, boot count tracking, automatic fallback
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use tokio::process::Command;
use tracing::{info, error, warn};
use crate::config;
use crate::partitions::PartitionManager;
use crate::Slot;

pub struct BootManager {
    config: Arc<RwLock<config::Config>>,
    partition_manager: Arc<PartitionManager>,
}

impl BootManager {
    pub fn new(config: Arc<RwLock<config::Config>>, partition_manager: Arc<PartitionManager>) -> Self {
        Self { config, partition_manager }
    }

    pub async fn get_boot_slot(&self) -> Slot {
        let cfg = self.config.read().await;
        if cfg.boot.default_slot == "a" { Slot::A } else { Slot::B }
    }

    pub async fn should_fallback(&self) -> bool {
        let boot_count_path = "/var/lib/kairos/boot_count";
        let count = match fs::read_to_string(boot_count_path).await {
            Ok(c) => c.trim().parse::<u32>().unwrap_or(0),
            Err(_) => 0,
        };
        let max_attempts = self.config.read().await.boot.max_boot_attempts;
        if count >= max_attempts {
            info!("Boot count {} exceeds max {} - triggering fallback", count, max_attempts);
            return true;
        }
        false
    }

    pub async fn increment_boot_count(&self) -> anyhow::Result<()> {
        let path = "/var/lib/kairos/boot_count";
        let count = match fs::read_to_string(path).await {
            Ok(c) => c.trim().parse::<u32>().unwrap_or(0) + 1,
            Err(_) => 1,
        };
        fs::write(path, count.to_string()).await?;
        info!("Boot count incremented to {}", count);
        Ok(())
    }

    pub async fn reset_boot_count(&self) -> anyhow::Result<()> {
        fs::write("/var/lib/kairos/boot_count", "0").await?;
        info!("Boot count reset to 0");
        Ok(())
    }

    pub async fn switch_slot(&self, target: &Slot) -> anyhow::Result<()> {
        // In production: use efibootmgr or grub-reboot
        let target_name = match target { Slot::A => "A", Slot::B => "B" };
        if let Err(e) = Command::new("grub-reboot").arg(target_name).status().await {
            warn!("grub-reboot failed (may not be installed): {}", e);
        }
        info!("Switching to slot {}", target_name);
        Ok(())
    }

    pub async fn mark_boot_successful(&self) -> anyhow::Result<()> {
        let active = self.partition_manager.get_active_slot().await;
        self.partition_manager.mark_slot_good(&active).await?;
        self.reset_boot_count().await?;
        info!("Boot marked successful for slot {:?}", active);
        Ok(())
    }
}
