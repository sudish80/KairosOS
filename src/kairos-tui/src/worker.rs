//! Background worker — render loop with fixed refresh rate
use crate::config;
use crate::framebuffer::Framebuffer;
use crate::layout::LayoutEngine;
use crate::telemetry::Telemetry;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct TuiWorker {
    config: Arc<RwLock<config::Config>>,
    layout_engine: Arc<LayoutEngine>,
    framebuffer: Arc<Framebuffer>,
    telemetry: Arc<Telemetry>,
}

impl TuiWorker {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        layout_engine: Arc<LayoutEngine>,
        framebuffer: Arc<Framebuffer>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        Self {
            config,
            layout_engine,
            framebuffer,
            telemetry,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let refresh_hz = self.config.read().await.general.refresh_rate_hz;
        let frame_duration = Duration::from_secs_f64(1.0 / refresh_hz as f64);
        info!(
            "TuiWorker started at {} Hz ({} ms per frame)",
            refresh_hz,
            frame_duration.as_millis()
        );

        let layout_engine = Arc::clone(&self.layout_engine);
        let framebuffer = Arc::clone(&self.framebuffer);
        let telemetry = Arc::clone(&self.telemetry);

        tokio::spawn(async move {
            let mut last_frame = Instant::now();
            loop {
                let frame_start = Instant::now();

                // Adaptive sleep to maintain target framerate
                let elapsed_since_last = frame_start.duration_since(last_frame);
                if elapsed_since_last < frame_duration {
                    tokio::time::sleep(frame_duration - elapsed_since_last).await;
                }
                last_frame = Instant::now();

                // Render frame
                match layout_engine.render().await {
                    Ok(buffer) => {
                        // Double-buffer swap
                        if let Err(e) = framebuffer.present(buffer).await {
                            error!("Frame present failed: {}", e);
                            telemetry.record_error();
                        }
                        // Actually swap the buffers
                        framebuffer.swap_buffers().await;
                    }
                    Err(e) => {
                        error!("Layout render failed: {}", e);
                        telemetry.record_error();
                    }
                }

                let frame_time = frame_start.elapsed();
                telemetry.record_frame(frame_time.as_nanos() as u64);

                // Warn if frame exceeds budget
                if frame_time > frame_duration && frame_time.as_millis() > 50 {
                    debug!(
                        "Frame took {} ms (budget: {} ms)",
                        frame_time.as_millis(),
                        frame_duration.as_millis()
                    );
                }
            }
        });

        Ok(())
    }
}
