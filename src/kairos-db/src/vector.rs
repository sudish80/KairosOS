use std::sync::Arc; use tokio::sync::RwLock; use crate::config;
pub struct VectorDatabase { config: Arc<RwLock<config::Config>>, dimension: usize }
impl VectorDatabase {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { let dim = config.blocking_read().vector.dimension; Self { config, dimension: dim } }
    pub async fn insert(&self, _id: &str, _vector: &[f32]) -> anyhow::Result<()> { Ok(()) }
    pub async fn search(&self, _query: &[f32], _k: usize) -> anyhow::Result<Vec<String>> { Ok(vec![]) }
}
