#![deny(unsafe_code)]
pub mod capture;
pub mod config;
pub mod detect;
pub mod error;
pub mod pipeline;
pub mod telemetry;
pub mod worker;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub capture: std::sync::Arc<capture::FrameCapture>,
    pub detector: std::sync::Arc<detect::ObjectDetector>,
    pub vision_pipeline: std::sync::Arc<pipeline::VisionPipeline>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry =
            std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let capture =
            std::sync::Arc::new(capture::FrameCapture::new(std::sync::Arc::clone(&config)));
        let detector =
            std::sync::Arc::new(detect::ObjectDetector::new(std::sync::Arc::clone(&config)));
        let vision_pipeline = std::sync::Arc::new(pipeline::VisionPipeline::new(
            std::sync::Arc::clone(&config),
            std::sync::Arc::clone(&capture),
            std::sync::Arc::clone(&detector),
        ));
        tracing::info!("kairos-vision AppState initialized");
        Ok(Self {
            config,
            telemetry,
            capture,
            detector,
            vision_pipeline,
        })
    }
}
