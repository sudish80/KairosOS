use std::sync::Arc; use tokio::sync::RwLock; use crate::config;
pub struct MavlinkBus { config: Arc<RwLock<config::Config>> }
impl MavlinkBus { pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn send_heartbeat(&self) -> anyhow::Result<()> { tracing::debug!("MAVLink heartbeat"); Ok(()) }
}
