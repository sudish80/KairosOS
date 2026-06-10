//! Update engine — OTA downloads, manifest parsing, staged rollouts, delta updates, scheduling
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::config;
use crate::partitions::PartitionManager;
use crate::verity::VerityManager;
use crate::boot::BootManager;
use crate::telemetry::Telemetry;
use crate::Slot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub format_version: u32,
    pub release_id: String,
    pub version: String,
    pub channel: String,
    pub staging_percentage: u32,
    pub images: Vec<ImageEntry>,
    pub deltas: Vec<DeltaEntry>,
    pub min_bootloader_version: String,
    pub signing_key_fingerprint: String,
    pub timestamp: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    pub target: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub compression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub from_version: String,
    pub to_version: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub manifest: Option<UpdateManifest>,
    pub eligible: bool,
    pub reason: String,
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub struct UpdateEngine {
    config: Arc<RwLock<config::Config>>,
    partition_manager: Arc<PartitionManager>,
    verity_manager: Arc<VerityManager>,
    boot_manager: Arc<BootManager>,
    telemetry: Arc<Telemetry>,
    http_client: reqwest::Client,
}

impl UpdateEngine {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        partition_manager: Arc<PartitionManager>,
        verity_manager: Arc<VerityManager>,
        boot_manager: Arc<BootManager>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .user_agent("kairos-updater/1.0")
            .build()
            .expect("Failed to create HTTP client");
        Self { config, partition_manager, verity_manager, boot_manager, telemetry, http_client }
    }

    /// Check for updates from the OTA server
    pub async fn check_for_update(&self) -> anyhow::Result<UpdateCheckResult> {
        let cfg = self.config.read().await;
        let device_id = if cfg.update.device_id.is_empty() {
            self.get_device_id().await?
        } else {
            cfg.update.device_id.clone()
        };
        let url = format!("{}/check?device_id={}&channel={}&version={}",
            cfg.update.server_url, device_id, cfg.update.channel, env!("CARGO_PKG_VERSION"));

        info!("Checking for update at {}", url);

        let resp = self.http_client.get(&url).send().await.map_err(|e| {
            anyhow::anyhow!("Update check request failed: {}", e)
        })?;

        if !resp.status().is_success() {
            return Ok(UpdateCheckResult {
                update_available: false, manifest: None,
                eligible: false, reason: format!("HTTP {}", resp.status()),
            });
        }

        let manifest: UpdateManifest = resp.json().await?;
        let eligible = self.is_eligible_for_staging(&manifest).await?;

        Ok(UpdateCheckResult {
            update_available: true,
            manifest: Some(manifest),
            eligible,
            reason: if eligible { "eligible".into() } else { "not in staged rollout group".into() },
        })
    }

    /// Download update image to download dir with SHA256 verification
    pub async fn download_update(&self, image_entry: &ImageEntry) -> anyhow::Result<String> {
        let cfg = self.config.read().await;
        let dest_dir = &cfg.update.download_dir;
        fs::create_dir_all(dest_dir).await?;

        let filename = image_entry.url.rsplit('/').next().unwrap_or("update.img");
        let dest_path = format!("{}/{}", dest_dir, filename);

        info!("Downloading {} -> {}", image_entry.url, dest_path);

        let resp = self.http_client.get(&image_entry.url).send().await?;
        let bytes = resp.bytes().await?;

        // Verify SHA256
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hash = hex_encode(&hasher.finalize());

        if actual_hash != image_entry.sha256 {
            return Err(anyhow::anyhow!("SHA256 mismatch: expected {}, got {}", image_entry.sha256, actual_hash));
        }

        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&dest_path).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;

        info!("Download verified, SHA256 match: {} ({} bytes)", actual_hash, bytes.len());
        self.telemetry.record_update_check(true);
        Ok(dest_path)
    }

    /// Write downloaded image to inactive slot
    pub async fn prepare_update(&self, image_path: &str) -> anyhow::Result<()> {
        let inactive = self.partition_manager.get_inactive_slot().await;
        let device = self.partition_manager.get_slot_device(&inactive).await;
        info!("Preparing update to slot {:?} (device: {})", inactive, device);

        if self.config.read().await.update.verify_signatures {
            self.verify_signature(image_path).await?;
        }

        let status = Command::new("dd")
            .args(["if", image_path, "of", &device, "bs", "4M", "status", "progress"])
            .status().await?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to write image to {}", device));
        }

        self.telemetry.record_update();
        info!("Update image written to slot {:?}", inactive);
        Ok(())
    }

    /// Apply a delta patch using bspatch
    pub async fn apply_delta_update(&self, delta: &DeltaEntry, base_path: &str) -> anyhow::Result<String> {
        let cfg = self.config.read().await;
        let delta_dir = &cfg.update.delta_cache_dir;
        fs::create_dir_all(delta_dir).await?;

        let delta_filename = delta.url.rsplit('/').next().unwrap_or("delta.patch");
        let delta_path = format!("{}/{}", delta_dir, delta_filename);
        let output_path = format!("{}/{}", delta_dir, "patched.img");

        // Download delta patch
        let resp = self.http_client.get(&delta.url).send().await?;
        let bytes = resp.bytes().await?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hash = hex_encode(&hasher.finalize());

        if actual_hash != delta.sha256 {
            return Err(anyhow::anyhow!("Delta SHA256 mismatch: expected {}, got {}", delta.sha256, actual_hash));
        }

        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&delta_path).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;

        // Apply bspatch
        info!("Applying delta patch: {} -> {}", base_path, output_path);
        let status = Command::new("bspatch")
            .args([base_path, &output_path, &delta_path])
            .status().await?;

        if !status.success() {
            return Err(anyhow::anyhow!("bspatch failed"));
        }

        Ok(output_path)
    }

    /// Finalize update — verity check, switch slot, mark successful
    pub async fn finalize_update(&self) -> anyhow::Result<()> {
        let inactive = self.partition_manager.get_inactive_slot().await;
        let device = self.partition_manager.get_slot_device(&inactive).await;

        let verified = self.verity_manager.verify_device(&device).await?;
        self.telemetry.record_verity_check(verified);

        if !verified {
            return Err(anyhow::anyhow!("Verity check failed for slot {:?}", inactive));
        }

        self.boot_manager.switch_slot(&inactive).await?;
        self.telemetry.record_update();
        info!("Update finalized, will boot slot {:?} next", inactive);
        Ok(())
    }

    /// Rollback to the other slot
    pub async fn rollback_update(&self) -> anyhow::Result<()> {
        let active = self.partition_manager.get_active_slot().await;
        let inactive = self.partition_manager.get_inactive_slot().await;
        info!("Rolling back from slot {:?} to {:?}", active, inactive);

        self.boot_manager.switch_slot(&inactive).await?;
        self.partition_manager.mark_slot_bad(&active).await?;
        self.telemetry.record_rollback();
        info!("Rollback complete, will boot slot {:?} next", inactive);
        Ok(())
    }

    /// Start the scheduled update check loop
    pub async fn start_scheduler(&self) {
        let interval_secs = self.config.read().await.update.check_interval_secs;
        info!("Starting update scheduler (interval: {}s)", interval_secs);
        let mut ticker = interval(Duration::from_secs(interval_secs));

        loop {
            ticker.tick().await;
            match self.check_for_update().await {
                Ok(result) => {
                    if result.update_available && result.eligible {
                        info!("Update available: {}",
                            result.manifest.as_ref().map(|m| &m.version[..]).unwrap_or("unknown"));
                        if self.config.read().await.update.auto_apply {
                            if let Some(manifest) = result.manifest {
                                if let Err(e) = self.apply_update_from_manifest(&manifest).await {
                                    error!("Auto-apply failed: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Update check failed: {}", e),
            }
        }
    }

    /// Full update from manifest: download -> prepare -> finalize
    pub async fn apply_update_from_manifest(&self, manifest: &UpdateManifest) -> anyhow::Result<()> {
        info!("Applying update {}", manifest.release_id);

        let current_version = env!("CARGO_PKG_VERSION");
        let delta = manifest.deltas.iter()
            .find(|d| d.from_version == current_version && d.to_version == manifest.version);

        let image_path = if let Some(d) = delta {
            let base_path = format!("{}/slot_{}.img", self.config.read().await.update.download_dir,
                match self.partition_manager.get_active_slot().await { Slot::A => "a", Slot::B => "b" });

            if PathBuf::from(&base_path).exists() {
                info!("Applying delta update from {} to {}", d.from_version, d.to_version);
                self.apply_delta_update(d, &base_path).await?
            } else {
                info!("No base image for delta, falling back to full download");
                let image = manifest.images.first().ok_or_else(|| anyhow::anyhow!("No images in manifest"))?;
                self.download_update(image).await?
            }
        } else {
            let image = manifest.images.first().ok_or_else(|| anyhow::anyhow!("No images in manifest"))?;
            self.download_update(image).await?
        };

        self.prepare_update(&image_path).await?;
        self.finalize_update().await
    }

    async fn get_device_id(&self) -> anyhow::Result<String> {
        if let Ok(machine_id) = fs::read_to_string("/etc/machine-id").await {
            return Ok(machine_id.trim().to_string());
        }
        if let Ok(dbus_id) = fs::read_to_string("/var/lib/dbus/machine-id").await {
            return Ok(dbus_id.trim().to_string());
        }
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn is_eligible_for_staging(&self, manifest: &UpdateManifest) -> anyhow::Result<bool> {
        let device_id = self.get_device_id().await?;
        let hash = Sha256::digest(device_id.as_bytes());
        let hash_prefix = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let pct = hash_prefix % 100;
        let threshold = if manifest.staging_percentage > 0 {
            manifest.staging_percentage
        } else {
            self.config.read().await.update.staging_percentage
        };
        Ok(pct < threshold)
    }

    async fn verify_signature(&self, image_path: &str) -> anyhow::Result<()> {
        let cfg = self.config.read().await;
        let sig_path = format!("{}.sig", image_path);

        if !PathBuf::from(&sig_path).exists() {
            warn!("Signature file not found: {}", sig_path);
            if cfg.update.verify_signatures {
                return Err(anyhow::anyhow!("Signature verification required but no signature found"));
            }
            return Ok(());
        }

        let status = Command::new("gpgv")
            .args(["--keyring", &cfg.update.gpg_keyring, &sig_path, image_path])
            .status().await?;

        if status.success() {
            info!("Signature verified for {}", image_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Signature verification failed for {}", image_path))
        }
    }

    pub async fn get_status(&self) -> anyhow::Result<UpdateStatus> {
        let cfg = self.config.read().await;
        let active = self.partition_manager.get_active_slot().await;
        let inactive = self.partition_manager.get_inactive_slot().await;
        let active_device = self.partition_manager.get_slot_device(&active).await;
        let inactive_device = self.partition_manager.get_slot_device(&inactive).await;

        Ok(UpdateStatus {
            active_slot: format!("{:?}", active),
            inactive_slot: format!("{:?}", inactive),
            active_device,
            inactive_device,
            server_url: cfg.update.server_url.clone(),
            channel: cfg.update.channel.clone(),
            auto_check: cfg.update.auto_check,
            auto_apply: cfg.update.auto_apply,
            check_interval_secs: cfg.update.check_interval_secs,
            staging_percentage: cfg.update.staging_percentage,
            updates_applied: self.telemetry.updates_applied(),
            rollbacks_performed: self.telemetry.rollbacks_performed(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub active_slot: String,
    pub inactive_slot: String,
    pub active_device: String,
    pub inactive_device: String,
    pub server_url: String,
    pub channel: String,
    pub auto_check: bool,
    pub auto_apply: bool,
    pub check_interval_secs: u64,
    pub staging_percentage: u32,
    pub updates_applied: u64,
    pub rollbacks_performed: u64,
}
