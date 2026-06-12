//! Layout engine — composites panes, status bar, tabs into final framebuffer
use crate::config;
use crate::multiplexer::Multiplexer;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LayoutEngine {
    config: Arc<RwLock<config::Config>>,
    multiplexer: Arc<Multiplexer>,
}

impl LayoutEngine {
    pub fn new(config: Arc<RwLock<config::Config>>, multiplexer: Arc<Multiplexer>) -> Self {
        Self {
            config,
            multiplexer,
        }
    }

    pub async fn render(&self) -> anyhow::Result<Vec<u32>> {
        let cfg = self.config.read().await;
        let (width, height) = (cfg.display.width, cfg.display.height);
        let mut buffer = vec![0u32; (width * height) as usize];

        let bg: u32 = 0x1E1E1E;
        buffer.fill(bg);

        // Render status bar at top
        if cfg.layout.status_bar {
            self.render_status_bar(&mut buffer, width, height).await;
        }

        // Render tab bar
        if cfg.layout.tab_bar {
            self.render_tab_bar(&mut buffer, width, height).await;
        }

        // Render active terminal content
        let panes = self.multiplexer.list_tabs().await;
        if !panes.is_empty() {
            let tab_bar_height = if cfg.layout.tab_bar { 30 } else { 0 };
            let status_bar_height = if cfg.layout.status_bar { 24 } else { 0 };
            let _term_height = height.saturating_sub(tab_bar_height + status_bar_height);
            // In production: render terminal cells to pixel buffer
        }

        Ok(buffer)
    }

    async fn render_status_bar(&self, buffer: &mut [u32], width: u32, height: u32) {
        // Draw status bar at bottom
        let bar_y = height - 24;
        for x in 0..width {
            buffer[(bar_y * width + x) as usize] = 0x007ACC;
        }
    }

    async fn render_tab_bar(&self, buffer: &mut [u32], width: u32, _height: u32) {
        // Draw tab bar at top
        for x in 0..width {
            buffer[x as usize] = 0x252526;
        }
    }
}
