use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct ObjectDetector {
    config: Arc<RwLock<config::Config>>,
}
impl ObjectDetector {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn detect(&self, _frame: &[u8]) -> anyhow::Result<Vec<Detection>> {
        Ok(vec![])
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Detection {
    pub label: String,
    pub confidence: f64,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
