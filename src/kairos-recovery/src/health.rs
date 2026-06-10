//! Health checker — real filesystem checks, mount verification, partition integrity
use crate::config;
use crate::partitions::PartitionManager;
use crate::Slot;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

static FSCK_PATH: &str = "/sbin/fsck";
static MOUNT_PATH: &str = "/bin/mount";
static BOOT_COUNT_PATH: &str = "/var/lib/kairos/boot_count";

pub struct HealthChecker {
    config: Arc<RwLock<config::Config>>,
    partition_manager: Arc<PartitionManager>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub timestamp: String,
    pub active_slot: String,
    pub active_slot_mounted: bool,
    pub inactive_slot_ok: bool,
    pub boot_count: u32,
    pub boot_count_ok: bool,
    pub free_space_bytes: u64,
    pub free_space_ok: bool,
    pub filesystem_ok: bool,
    pub verity_enabled: bool,
    pub recovery_partition_mounted: bool,
    pub overall: String,
    pub details: Vec<String>,
}

impl HealthChecker {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        partition_manager: Arc<PartitionManager>,
    ) -> Self {
        Self {
            config,
            partition_manager,
        }
    }

    pub async fn check_health(&self) -> anyhow::Result<HealthReport> {
        let active = self.partition_manager.get_active_slot().await;
        let active_name = match active {
            Slot::A => "A",
            Slot::B => "B",
        };
        let mut details = Vec::new();

        // 1. Check if active partition device exists
        let active_dev = self.partition_manager.get_slot_device(&active).await;
        let active_dev_exists = std::path::Path::new(&active_dev).exists();
        if !active_dev_exists {
            details.push(format!("Active slot device {} does not exist", active_dev));
        }

        // 2. Try mounting active slot to verify it's functional
        let active_mounted = match self.partition_manager.mount_slot(&active).await {
            Ok(mp) => {
                self.partition_manager.unmount_slot(&mp).await?;
                details.push(format!("Slot {} mounted and verified OK", active_name));
                true
            }
            Err(e) => {
                details.push(format!("Slot {} mount failed: {}", active_name, e));
                false
            }
        };

        // 3. Check inactive slot
        let inactive = self.partition_manager.get_inactive_slot().await;
        let inactive_dev = self.partition_manager.get_slot_device(&inactive).await;
        let inactive_exists = std::path::Path::new(&inactive_dev).exists();
        if inactive_exists {
            details.push(format!("Inactive slot device {} present", inactive_dev));
        } else {
            details.push("Inactive slot device not found — may be first boot".into());
        }

        // 4. Check free space on active slot
        let min_free = self.config.read().await.partitions.min_free_bytes;
        let free_space = self
            .partition_manager
            .check_free_space(&active)
            .await
            .unwrap_or(0);
        let free_space_ok = free_space >= min_free;
        details.push(format!(
            "Free space: {} MB (min: {} MB)",
            free_space / 1024 / 1024,
            min_free / 1024 / 1024
        ));

        // 5. Filesystem integrity check (read-only)
        let filesystem_ok = if active_dev_exists {
            match Command::new(FSCK_PATH)
                .args(["-n", "-q", &active_dev])
                .output()
                .await
            {
                Ok(out) => {
                    let ok = out.status.success();
                    if !ok {
                        details.push(format!("fsck reported issues on {}", active_dev));
                    }
                    ok
                }
                Err(_) => {
                    details.push("fsck not available, skipping".into());
                    true
                }
            }
        } else {
            false
        };

        // 6. Boot count check
        let boot_count = match fs::read_to_string(BOOT_COUNT_PATH).await {
            Ok(c) => c.trim().parse::<u32>().unwrap_or(0),
            Err(_) => 0,
        };
        let max_boot = self.config.read().await.boot.max_boot_attempts;
        let boot_count_ok = boot_count < max_boot;
        details.push(format!("Boot count: {}/{}", boot_count, max_boot));

        // 7. Check recovery partition
        let recovery_dev = &self.config.read().await.recovery.recovery_partition;
        let recovery_mounted = std::path::Path::new(recovery_dev).exists();
        if !recovery_mounted {
            details.push(format!("Recovery partition {} not found", recovery_dev));
        }

        // 8. Check if verity is setup
        let verity_dev = &self.config.read().await.verity.hash_device;
        let verity_enabled = std::path::Path::new(verity_dev).exists();

        // Overall assessment
        let all_ok =
            active_dev_exists && active_mounted && filesystem_ok && free_space_ok && boot_count_ok;
        let overall = if all_ok {
            "healthy"
        } else if active_dev_exists && active_mounted {
            "degraded"
        } else {
            "critical"
        };

        info!("Health check complete: overall={}", overall);

        Ok(HealthReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            active_slot: active_name.into(),
            active_slot_mounted: active_mounted,
            inactive_slot_ok: inactive_exists,
            boot_count,
            boot_count_ok,
            free_space_bytes: free_space,
            free_space_ok,
            filesystem_ok,
            verity_enabled,
            recovery_partition_mounted: recovery_mounted,
            overall: overall.into(),
            details,
        })
    }
}
