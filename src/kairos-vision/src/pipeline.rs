use std::sync::Arc; use tokio::sync::RwLock; use crate::config; use crate::capture::FrameCapture; use crate::detect::ObjectDetector;
pub struct VisionPipeline { config: Arc<RwLock<config::Config>>, _capture: Arc<FrameCapture>, _detector: Arc<ObjectDetector> }
impl VisionPipeline {
    pub fn new(config: Arc<RwLock<config::Config>>, capture: Arc<FrameCapture>, detector: Arc<ObjectDetector>) -> Self { Self { config, _capture: capture, _detector: detector } }
    pub async fn process_frame(&self) -> anyhow::Result<()> { tracing::debug!("Processing frame"); Ok(()) }
}
