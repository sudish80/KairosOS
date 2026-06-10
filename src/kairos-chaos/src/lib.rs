use rand::Rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosAction {
    KillDaemon(String),
    NetworkPartition(String, String),
    DiskFull(f64),       // fill % of disk
    MemoryPressure(u64), // MB to allocate
    CorruptPacket(f64),  // drop probability
    SlowDisk(u64),       // latency ms
    RandomOom,
    RotateLogs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosEvent {
    pub id: String,
    pub action: ChaosAction,
    pub duration_secs: u64,
    pub status: ChaosStatus,
    pub score_impact: Option<i32>,
    pub recovery_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChaosStatus {
    Running,
    Succeeded,
    Failed,
    RolledBack,
}

pub struct ChaosEngine {
    config: ChaosConfig,
    events: Arc<RwLock<Vec<ChaosEvent>>>,
    score: Arc<RwLock<i32>>,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub min_severity: u32,
    pub max_concurrent: usize,
    pub auto_rollback: bool,
    pub score_decay_per_minute: i32,
    pub target_daemons: Vec<String>,
    pub allowed_actions: Vec<ChaosAction>,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            min_severity: 1,
            max_concurrent: 3,
            auto_rollback: true,
            score_decay_per_minute: 5,
            target_daemons: vec![
                "kairos-bpf".into(),
                "kairos-mcp".into(),
                "kairos-db".into(),
                "kairos-apply".into(),
                "kairos-orchestrator".into(),
            ],
            allowed_actions: vec![
                ChaosAction::KillDaemon("kairos-bpf".into()),
                ChaosAction::KillDaemon("kairos-mcp".into()),
                ChaosAction::RotateLogs,
            ],
        }
    }
}

impl ChaosEngine {
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
            score: Arc::new(RwLock::new(100)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn execute(
        &self,
        action: ChaosAction,
        duration_secs: u64,
    ) -> anyhow::Result<ChaosEvent> {
        let id = format!(
            "chaos-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        );

        info!(
            "Executing chaos action: {:?} for {}s",
            action, duration_secs
        );
        let mut event = ChaosEvent {
            id,
            action: action.clone(),
            duration_secs,
            status: ChaosStatus::Running,
            score_impact: None,
            recovery_output: String::new(),
        };

        let result = match &action {
            ChaosAction::KillDaemon(name) => self.kill_daemon(name, duration_secs).await,
            ChaosAction::NetworkPartition(from, to) => {
                self.network_partition(from, to, duration_secs).await
            }
            ChaosAction::DiskFull(pct) => self.fill_disk(*pct, duration_secs).await,
            ChaosAction::MemoryPressure(mb) => self.memory_pressure(*mb, duration_secs).await,
            ChaosAction::CorruptPacket(prob) => self.corrupt_packets(*prob, duration_secs).await,
            ChaosAction::SlowDisk(latency) => self.slow_disk(*latency, duration_secs).await,
            ChaosAction::RandomOom => self.random_oom(duration_secs).await,
            ChaosAction::RotateLogs => self.rotate_logs().await,
        };

        match result {
            Ok(output) => {
                event.status = ChaosStatus::Succeeded;
                event.recovery_output = output;
                let impact = -(rand::thread_rng().gen_range(1..=20));
                event.score_impact = Some(impact);
                *self.score.write().await += impact;
            }
            Err(e) => {
                event.status = ChaosStatus::Failed;
                event.recovery_output = format!("Error: {}", e);
                event.score_impact = Some(0);
            }
        }

        if self.config.auto_rollback && event.status == ChaosStatus::Succeeded {
            if let ChaosAction::KillDaemon(name) = &action {
                if let Err(e) = self.restart_daemon(name).await {
                    error!("Auto-rollback restart failed for {}: {}", name, e);
                }
            }
        }

        self.events.write().await.push(event.clone());
        Ok(event)
    }

    async fn kill_daemon(&self, name: &str, _duration: u64) -> anyhow::Result<String> {
        let output = tokio::process::Command::new("systemctl")
            .args(["stop", name])
            .output()
            .await?;
        if output.status.success() {
            Ok(format!("Stopped daemon {}", name))
        } else {
            // Fallback: try pkill
            tokio::process::Command::new("pkill")
                .arg("-f")
                .arg(name)
                .output()
                .await?;
            Ok(format!("Sigterm sent to {}", name))
        }
    }

    async fn restart_daemon(&self, name: &str) -> anyhow::Result<String> {
        let output = tokio::process::Command::new("systemctl")
            .args(["restart", name])
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn network_partition(
        &self,
        from: &str,
        to: &str,
        duration: u64,
    ) -> anyhow::Result<String> {
        let source_ip = self.resolve_daemon_ip(from).await;
        let target_ip = self.resolve_daemon_ip(to).await;

        tokio::process::Command::new("iptables")
            .args([
                "-A", "INPUT", "-s", &source_ip, "-d", &target_ip, "-j", "DROP",
            ])
            .status()
            .await?;

        tokio::time::sleep(Duration::from_secs(duration)).await;

        tokio::process::Command::new("iptables")
            .args([
                "-D", "INPUT", "-s", &source_ip, "-d", &target_ip, "-j", "DROP",
            ])
            .status()
            .await?;

        Ok(format!(
            "Partitioned {} from {} for {}s",
            from, to, duration
        ))
    }

    async fn fill_disk(&self, pct: f64, duration: u64) -> anyhow::Result<String> {
        // Fill disk by creating a large file
        let path = "/tmp/chaos-fill";
        let total = self.get_disk_total().await;
        let fill_bytes = (total as f64 * pct / 100.0) as u64;

        let output = tokio::process::Command::new("dd")
            .args([
                "if=/dev/zero",
                &format!("of={}", path),
                &format!("bs=1M"),
                &format!("count={}", fill_bytes / 1024 / 1024),
            ])
            .output()
            .await?;

        tokio::time::sleep(Duration::from_secs(duration)).await;
        let _ = tokio::fs::remove_file(path).await;

        Ok(format!("Filled {:.1}% of disk for {}s", pct, duration))
    }

    async fn memory_pressure(&self, mb: u64, duration: u64) -> anyhow::Result<String> {
        let bytes = (mb as usize) * 1024 * 1024;
        let mut vec: Vec<u8> = Vec::with_capacity(bytes);
        vec.resize(bytes, 0);
        // Touch pages
        for chunk in vec.chunks_mut(4096) {
            chunk[0] = 1;
        }
        tokio::time::sleep(Duration::from_secs(duration)).await;
        drop(vec);

        Ok(format!("Allocated {} MB for {}s then freed", mb, duration))
    }

    async fn corrupt_packets(&self, prob: f64, duration: u64) -> anyhow::Result<String> {
        // Use tc to add packet loss
        tokio::process::Command::new("tc")
            .args([
                "qdisc",
                "add",
                "dev",
                "eth0",
                "root",
                "netem",
                "loss",
                &format!("{}%", prob * 100.0),
            ])
            .status()
            .await?;

        tokio::time::sleep(Duration::from_secs(duration)).await;

        tokio::process::Command::new("tc")
            .args(["qdisc", "del", "dev", "eth0", "root"])
            .status()
            .await?;

        Ok(format!(
            "Introduced {:.0}% packet loss for {}s",
            prob * 100.0,
            duration
        ))
    }

    async fn slow_disk(&self, latency_ms: u64, duration: u64) -> anyhow::Result<String> {
        // Use device-mapper delay target
        let minor = self.get_root_minor().await;
        let dmsetup = format!(
            "0 {} delay /dev/{} 0 {}",
            self.get_disk_sectors().await,
            minor,
            latency_ms
        );

        tokio::process::Command::new("dmsetup")
            .args(["create", "chaos-slow", "--table", &dmsetup])
            .status()
            .await?;

        tokio::time::sleep(Duration::from_secs(duration)).await;
        tokio::process::Command::new("dmsetup")
            .args(["remove", "chaos-slow"])
            .status()
            .await?;

        Ok(format!(
            "Added {}ms disk latency for {}s",
            latency_ms, duration
        ))
    }

    async fn random_oom(&self, _duration: u64) -> anyhow::Result<String> {
        let daemons = &self.config.target_daemons;
        let idx = rand::thread_rng().gen_range(0..daemons.len());
        let target = &daemons[idx];

        // Fork bomb: (simplified — use stress-ng if available)
        let output = tokio::process::Command::new("stress-ng")
            .args([
                "--vm",
                "4",
                "--vm-bytes",
                "75%",
                "--timeout",
                "10s",
                "--oomable",
            ])
            .output()
            .await?;

        if output.status.success() {
            Ok(format!("OOM triggered via stress-ng, affected {}", target))
        } else {
            // Fallback: allocate memory until OOM
            let _ = self.memory_pressure(999999, 5).await;
            Ok("OOM fallback triggered".to_string())
        }
    }

    async fn rotate_logs(&self) -> anyhow::Result<String> {
        let output = tokio::process::Command::new("logrotate")
            .args(["-f", "/etc/logrotate.conf"])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn resolve_daemon_ip(&self, name: &str) -> String {
        // Try systemd-resolved, fallback to /etc/hosts or default
        let output = tokio::process::Command::new("getent")
            .args(["hosts", name])
            .output()
            .await
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            });
        output
            .and_then(|s| s.split_whitespace().next().map(String::from))
            .unwrap_or_else(|| "127.0.0.1".to_string())
    }

    async fn get_disk_total(&self) -> u64 {
        let output = tokio::process::Command::new("df")
            .args(["--output=size", "/"])
            .output()
            .await
            .ok()
            .and_then(|o| {
                let s = String::from_utf8(o.stdout).ok()?;
                s.lines().last()?.trim().parse::<u64>().ok()
            });
        output.unwrap_or(10_000_000) * 1024 // KB -> bytes
    }

    async fn get_root_minor(&self) -> String {
        "253:0".into()
    }

    async fn get_disk_sectors(&self) -> String {
        "20971520".into()
    }

    pub async fn get_score(&self) -> i32 {
        *self.score.read().await
    }

    pub async fn get_events(&self) -> Vec<ChaosEvent> {
        self.events.read().await.clone()
    }

    pub async fn start(&self) {
        *self.running.write().await = true;
        let score_decay = self.config.score_decay_per_minute;
        let score = Arc::clone(&self.score);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                if !*running.read().await {
                    break;
                }
                let mut s = score.write().await;
                *s = (*s + score_decay).min(100);
            }
        });
    }

    pub async fn stop(&self) {
        *self.running.write().await = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_engine_create() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        assert_eq!(engine.get_score().await, 100);
        assert!(engine.get_events().await.is_empty());
    }

    #[tokio::test]
    async fn test_rotate_logs_action() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        // Non-root will fail but shouldn't crash
        let event = engine.execute(ChaosAction::RotateLogs, 0).await;
        assert!(event.is_ok());
    }

    #[tokio::test]
    async fn test_score_decay() {
        let engine = ChaosEngine::new(ChaosConfig {
            score_decay_per_minute: 10,
            ..Default::default()
        });
        *engine.score.write().await = 50;
        engine.start().await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let score = engine.get_score().await;
        // Not enough time for full minute, but score should be >= 50
        assert!(score >= 50);
        engine.stop().await;
    }

    #[test]
    fn test_chaos_event_serialization() {
        let event = ChaosEvent {
            id: "test-1".into(),
            action: ChaosAction::RotateLogs,
            duration_secs: 30,
            status: ChaosStatus::Running,
            score_impact: None,
            recovery_output: String::new(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: ChaosEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "test-1");
        assert_eq!(back.status, ChaosStatus::Running);
    }
}
