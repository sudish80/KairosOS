//! DRM/KMS manager — direct rendering, page flip, mode setting
use crate::config;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct DrmManager {
    config: Arc<RwLock<config::Config>>,
    drm_fd: Option<i32>,
    crtc_id: Option<u32>,
    connector_id: Option<u32>,
}

impl DrmManager {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        info!("DrmManager initialized");
        Ok(Self {
            config,
            drm_fd: None,
            crtc_id: None,
            connector_id: None,
        })
    }

    pub async fn open(&self) -> anyhow::Result<()> {
        let device = self.config.read().await.display.drm_device.clone();
        // In production: open DRM device, get resources
        info!("DRM device: {}", device);
        Ok(())
    }

    pub async fn set_mode(&self, width: u32, height: u32, bpp: u32) -> anyhow::Result<()> {
        let status = Command::new("fbset")
            .args([
                "-xres",
                &width.to_string(),
                "-yres",
                &height.to_string(),
                "-depth",
                &bpp.to_string(),
            ])
            .status()
            .await;

        match status {
            Ok(s) if s.success() => info!("Display mode set: {}x{}x{}", width, height, bpp),
            _ => debug!("fbset not available, using current mode"),
        }
        Ok(())
    }

    pub async fn page_flip(&self, fb_id: u32) -> anyhow::Result<()> {
        // In production: drmModePageFlip
        Ok(())
    }

    pub async fn get_connector_status(&self) -> bool {
        // In production: drmModeGetConnector
        true
    }
}
