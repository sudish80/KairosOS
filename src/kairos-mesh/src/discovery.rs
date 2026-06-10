use std::sync::Arc; use tokio::sync::RwLock; use tokio::net::UdpSocket;
use tracing::{info, error, debug}; use crate::config;
pub struct NodeDiscovery { config: Arc<RwLock<config::Config>> }
impl NodeDiscovery {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn start_discovery(&self) -> anyhow::Result<()> {
        let cfg = self.config.read().await;
        let interval = cfg.discovery.interval_secs;
        let seeds = cfg.discovery.seeds.clone();
        drop(cfg);
        info!("Discovery started, interval {}s, seeds: {:?}", interval, seeds);
        let socket = UdpSocket::bind("0.0.0.0:51821").await?;
        tokio::spawn(async move {
            loop {
                // Broadcast discovery beacons
                for seed in &seeds {
                    if let Err(e) = socket.send_to(b"KAIROS_DISCOVERY", seed).await {
                        debug!("Discovery send to {} failed: {}", seed, e);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            }
        });
        Ok(())
    }
}
