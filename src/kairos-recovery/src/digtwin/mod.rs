use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command as TokioCommand;
use tracing::info;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: f64,
    pub path: PathBuf,
    pub checksum: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinConfig {
    pub snapshot_dir: PathBuf,
    pub sandbox_dir: PathBuf,
    pub bubblewrap_path: String,
    pub max_snapshots: usize,
    pub packages_required: Vec<String>,
}

impl Default for TwinConfig {
    fn default() -> Self {
        Self {
            snapshot_dir: PathBuf::from("/var/lib/kairos/digital-twin/snapshots"),
            sandbox_dir: PathBuf::from("/var/lib/kairos/digital-twin/sandbox"),
            bubblewrap_path: "bwrap".into(),
            max_snapshots: 10,
            packages_required: vec!["bubblewrap".into()],
        }
    }
}

pub struct DigitalTwin {
    config: TwinConfig,
    current: Arc<RwLock<Option<Snapshot>>>,
}

impl DigitalTwin {
    pub fn new(config: TwinConfig) -> Self {
        Self {
            config,
            current: Arc::new(RwLock::new(None)),
        }
    }

    pub fn is_bubblewrap_available() -> bool {
        Command::new("bwrap")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn verify_packages(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for pkg in &self.config.packages_required {
            match pkg.as_str() {
                "bubblewrap" => {
                    if !DigitalTwin::is_bubblewrap_available() {
                        missing.push(pkg.clone());
                    }
                }
                other => {
                    let check = Command::new("which").arg(other).output();
                    if check.map(|o| !o.status.success()).unwrap_or(true) {
                        missing.push(pkg.clone());
                    }
                }
            }
        }
        missing
    }

    pub async fn create_snapshot(&self, path: &Path) -> anyhow::Result<Snapshot> {
        std::fs::create_dir_all(&self.config.snapshot_dir)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs_f64();
        let id = format!("twin-{:.0}", timestamp);
        let snapshot_path = self.config.snapshot_dir.join(format!("{}.tar.zst", id));

        let status = TokioCommand::new("tar")
            .arg("--zstd")
            .arg("-cf")
            .arg(&snapshot_path)
            .arg("-C")
            .arg(path)
            .arg(".")
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("tar snapshot failed");
        }

        let metadata = std::fs::metadata(&snapshot_path)?;
        let mut hasher = Sha256::new();
        let data = std::fs::read(&snapshot_path)?;
        hasher.update(&data);
        let checksum = format!("{:x}", hasher.finalize());

        let snapshot = Snapshot {
            id,
            timestamp,
            path: snapshot_path,
            checksum,
            size_bytes: metadata.len(),
        };

        *self.current.write().await = Some(snapshot.clone());
        self.prune_snapshots().await;
        Ok(snapshot)
    }

    pub async fn prune_snapshots(&self) {
        let mut entries: Vec<_> = std::fs::read_dir(&self.config.snapshot_dir)
            .map(|d| d.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_default();
        entries.sort_by_key(|e| e.path());

        while entries.len() > self.config.max_snapshots {
            if let Some(oldest) = entries.first() {
                let path = oldest.path();
                if std::fs::remove_file(&path).is_ok() {
                    info!("Pruned old snapshot: {:?}", path);
                }
                entries.remove(0);
            }
        }
    }

    pub async fn test_ota_in_sandbox(&self, ota_image: &Path) -> anyhow::Result<(bool, String)> {
        if !DigitalTwin::is_bubblewrap_available() {
            return Err(anyhow::anyhow!("bubblewrap not installed"));
        }

        let sandbox = &self.config.sandbox_dir;
        std::fs::create_dir_all(sandbox)?;

        let output = TokioCommand::new(&self.config.bubblewrap_path)
            .arg("--ro-bind")
            .arg("/usr")
            .arg("/usr")
            .arg("--ro-bind")
            .arg(ota_image)
            .arg("/ota-image")
            .arg("--proc")
            .arg("/proc")
            .arg("--dev")
            .arg("/dev")
            .arg("--bind")
            .arg(sandbox)
            .arg("/mnt")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg("ls /ota-image && dd if=/ota-image of=/mnt/test.img bs=1M count=10 2>&1")
            .output()
            .await?;

        let log = String::from_utf8_lossy(&output.stdout).to_string();
        let err = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((output.status.success(), format!("{}\n{}", log, err)))
    }

    pub async fn get_current(&self) -> Option<Snapshot> {
        self.current.read().await.clone()
    }

    pub async fn clone_environment(&self, source: &Path) -> anyhow::Result<PathBuf> {
        let id = format!("env-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs());
        let target = self.config.sandbox_dir.join(&id);
        std::fs::create_dir_all(&target)?;

        let status = TokioCommand::new("cp")
            .arg("-a")
            .arg(source)
            .arg(&target)
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("cp -a failed");
        }

        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_snapshot() {
        let tmp = TempDir::new().unwrap();
        let src = TempDir::new().unwrap();
        fs::write(src.path().join("test.txt"), b"hello world").unwrap();

        let twin = DigitalTwin::new(TwinConfig {
            snapshot_dir: tmp.path().join("snapshots"),
            sandbox_dir: tmp.path().join("sandbox"),
            ..Default::default()
        });
        let snap = twin.create_snapshot(src.path()).await.unwrap();
        assert!(snap.path.exists());
        assert!(!snap.checksum.is_empty());
        assert!(snap.size_bytes > 0);
    }

    #[tokio::test]
    async fn test_max_snapshots() {
        let tmp = TempDir::new().unwrap();
        let src = TempDir::new().unwrap();
        let twin = DigitalTwin::new(TwinConfig {
            snapshot_dir: tmp.path().join("snapshots"),
            sandbox_dir: tmp.path().join("sandbox"),
            max_snapshots: 3,
            ..Default::default()
        });

        for _ in 0..5 {
            let s = TempDir::new().unwrap();
            fs::write(s.path().join("f.txt"), b"data").unwrap();
            twin.create_snapshot(s.path()).await.unwrap();
        }

        let count = std::fs::read_dir(tmp.path().join("snapshots"))
            .map(|d| d.count())
            .unwrap_or(0);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_twin_config_default() {
        let c = TwinConfig::default();
        assert_eq!(c.max_snapshots, 10);
    }

    #[tokio::test]
    async fn test_verify_missing_bwrap() {
        let twin = DigitalTwin::new(TwinConfig::default());
        let missing = twin.verify_packages().await;
        if !DigitalTwin::is_bubblewrap_available() {
            assert!(missing.contains(&"bubblewrap".to_string()));
        }
    }
}
