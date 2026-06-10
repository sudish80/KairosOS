use std::sync::Arc; use tokio::sync::RwLock; use tokio::process::Command;
use tracing::{info, error}; use crate::config;
static WG_QUICK: &str = "/usr/bin/wg-quick";
pub struct WireGuardManager { config: Arc<RwLock<config::Config>> }
impl WireGuardManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn bring_up(&self) -> anyhow::Result<()> {
        let iface = self.config.read().await.wg.interface.clone();
        let status = Command::new(WG_QUICK).args(["up", &iface]).status().await?;
        if status.success() { info!("WireGuard {} up", iface); Ok(()) } else { Err(anyhow::anyhow!("Failed to bring up {}", iface)) }
    }
    pub async fn bring_down(&self) -> anyhow::Result<()> {
        let iface = self.config.read().await.wg.interface.clone();
        Command::new(WG_QUICK).args(["down", &iface]).status().await?;
        info!("WireGuard {} down", iface); Ok(())
    }
    pub async fn add_peer(&self, peer: &str) -> anyhow::Result<()> {
        let iface = self.config.read().await.wg.interface.clone();
        Command::new("wg").args(["set", &iface, "peer", peer, "allowed-ips", "0.0.0.0/0"]).status().await?;
        info!("Peer added: {}", peer); Ok(())
    }
}
