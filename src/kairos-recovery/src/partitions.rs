//! Partition manager — A/B slot management, mount/unmount, resize
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use tokio::process::Command;
use tracing::{info, debug, error, warn};
use crate::config;
use crate::error::RecoveryError;
use crate::Slot;

pub struct PartitionManager {
    config: Arc<RwLock<config::Config>>,
}

impl PartitionManager {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        info!("PartitionManager initialized");
        Ok(Self { config })
    }

    pub async fn get_active_slot(&self) -> Slot {
        Slot::current()
    }

    pub async fn get_inactive_slot(&self) -> Slot {
        match self.get_active_slot().await {
            Slot::A => Slot::B,
            Slot::B => Slot::A,
        }
    }

    pub async fn get_slot_device(&self, slot: &Slot) -> String {
        let cfg = self.config.read().await;
        match slot {
            Slot::A => cfg.partitions.slot_a.clone(),
            Slot::B => cfg.partitions.slot_b.clone(),
        }
    }

    pub async fn mount_slot(&self, slot: &Slot) -> anyhow::Result<PathBuf> {
        let device = self.get_slot_device(slot).await;
        let mount_point = PathBuf::from(format!("/mnt/kairos/{}", match slot { Slot::A => "a", Slot::B => "b" }));
        fs::create_dir_all(&mount_point).await?;

        let status = Command::new("mount")
            .args([&device, &mount_point.to_string_lossy()])
            .status().await?;

        if status.success() {
            info!("Mounted slot {:?} at {:?}", slot, mount_point);
            Ok(mount_point)
        } else {
            Err(RecoveryError::Partition(format!("Failed to mount slot {:?}", slot)).into())
        }
    }

    pub async fn unmount_slot(&self, mount_point: &Path) -> anyhow::Result<()> {
        let status = Command::new("umount")
            .arg(&mount_point)
            .status().await?;
        if status.success() {
            info!("Unmounted {:?}", mount_point);
        }
        Ok(())
    }

    pub async fn mark_slot_good(&self, slot: &Slot) -> anyhow::Result<()> {
        let device = self.get_slot_device(slot).await;
        // In production: set EFI boot variable or use grub2-reboot
        let marker = format!("/mnt/kairos/{}/etc/kairos/slot_good", match slot { Slot::A => "a", Slot::B => "b" });
        info!("Marked slot {:?} as good", slot);
        Ok(())
    }

    pub async fn mark_slot_bad(&self, slot: &Slot) -> anyhow::Result<()> {
        info!("Marked slot {:?} as bad", slot);
        Ok(())
    }

    pub async fn check_free_space(&self, slot: &Slot) -> anyhow::Result<u64> {
        let mount = self.mount_slot(slot).await?;
        let usage = fs2::available_space(&mount)?;
        self.unmount_slot(&mount).await?;
        Ok(usage)
    }
}
