use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub partitions: PartitionConfig,
    pub verity: VerityConfig,
    pub boot: BootConfig,
    pub update: UpdateConfig,
    pub recovery: RecoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    pub slot_a: String,
    pub slot_b: String,
    pub data_partition: String,
    pub efi_partition: String,
    pub min_free_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerityConfig {
    pub hash_device: String,
    pub root_hash_file: String,
    pub verify_on_boot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    pub timeout_secs: u32,
    pub default_slot: String,
    pub max_boot_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub download_dir: String,
    pub verify_signatures: bool,
    pub gpg_keyring: String,
    pub max_update_attempts: u32,
    pub server_url: String,
    pub channel: String,
    pub staging_percentage: u32,
    pub auto_check: bool,
    pub auto_apply: bool,
    pub check_interval_secs: u64,
    pub manifest_path: String,
    pub delta_cache_dir: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub shell_enabled: bool,
    pub shell_path: String,
    pub recovery_partition: String,
    pub network_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                log_level: "info".into(),
                data_dir: "/var/lib/kairos/recovery".into(),
            },
            partitions: PartitionConfig {
                slot_a: "/dev/disk/by-label/KAIROS_A".into(),
                slot_b: "/dev/disk/by-label/KAIROS_B".into(),
                data_partition: "/dev/disk/by-label/KAIROS_DATA".into(),
                efi_partition: "/dev/disk/by-partlabel/EFI".into(),
                min_free_bytes: 512 * 1024 * 1024,
            },
            verity: VerityConfig {
                hash_device: "/dev/disk/by-label/KAIROS_HASH".into(),
                root_hash_file: "/etc/kairos/root_hash.sig".into(),
                verify_on_boot: true,
            },
            boot: BootConfig {
                timeout_secs: 5,
                default_slot: "a".into(),
                max_boot_attempts: 3,
            },
            update: UpdateConfig {
                download_dir: "/var/lib/kairos/updates".into(),
                verify_signatures: true,
                gpg_keyring: "/etc/kairos/gpg".into(),
                max_update_attempts: 3,
                server_url: "https://updates.kairosos.org/v1".into(),
                channel: "stable".into(),
                staging_percentage: 10,
                auto_check: true,
                auto_apply: false,
                check_interval_secs: 86400,
                manifest_path: "/var/lib/kairos/updates/manifest.json".into(),
                delta_cache_dir: "/var/lib/kairos/updates/delta".into(),
                device_id: "".into(),
            },
            recovery: RecoveryConfig {
                shell_enabled: true,
                shell_path: "/bin/bash".into(),
                recovery_partition: "/dev/disk/by-label/KAIROS_RECOVERY".into(),
                network_enabled: true,
            },
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
