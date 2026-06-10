use std::sync::Arc; use tokio::sync::RwLock; use tracing::info; use crate::config; use crate::sequence::SequenceEngine; use crate::alignment::SequenceAligner;
pub struct BioPipeline { config: Arc<RwLock<config::Config>>, _seq: Arc<SequenceEngine>, _align: Arc<SequenceAligner> }
impl BioPipeline { pub fn new(config: Arc<RwLock<config::Config>>, seq: Arc<SequenceEngine>, align: Arc<SequenceAligner>) -> Self { Self{config, _seq: seq, _align: align} }
    pub async fn analyze(&self) -> anyhow::Result<()> { info!("Bio pipeline analysis"); Ok(()) }
}
