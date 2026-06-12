//! Framebuffer — double-buffered pixel buffer with blitting and composition
use crate::config;
use crate::error::TuiError;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Framebuffer {
    config: Arc<RwLock<config::Config>>,
    front_buffer: Arc<RwLock<Vec<u32>>>,
    back_buffer: Arc<RwLock<Vec<u32>>>,
    width: u32,
    height: u32,
    fd: Option<i32>,
}

impl Framebuffer {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        let (width, height) = {
            let cfg = config.read().await;
            (cfg.display.width, cfg.display.height)
        };
        let size = (width * height) as usize;
        Ok(Self {
            config,
            front_buffer: Arc::new(RwLock::new(vec![0u32; size])),
            back_buffer: Arc::new(RwLock::new(vec![0u32; size])),
            width,
            height,
            fd: None,
        })
    }

    pub async fn clear(&self, color: u32) {
        let mut back = self.back_buffer.write().await;
        back.fill(color);
    }

    pub async fn draw_pixel(&self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        let mut back = self.back_buffer.write().await;
        if idx < back.len() {
            back[idx] = color;
        }
    }

    pub async fn draw_rect(&self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let mut back = self.back_buffer.write().await;
        for dy in 0..h.min(self.height.saturating_sub(y)) {
            for dx in 0..w.min(self.width.saturating_sub(x)) {
                let idx = ((y + dy) * self.width + (x + dx)) as usize;
                if idx < back.len() {
                    back[idx] = color;
                }
            }
        }
    }

    pub async fn draw_text(
        &self,
        x: u32,
        y: u32,
        text: &str,
        color: u32,
        font_width: u32,
        font_height: u32,
    ) {
        for (i, ch) in text.chars().enumerate() {
            let chx = x + (i as u32 * font_width);
            if chx + font_width > self.width {
                break;
            }
            // In production: blit glyph bitmap from font
            self.draw_rect(chx, y, font_width, font_height, color).await;
        }
    }

    pub async fn swap_buffers(&self) {
        let mut front = self.front_buffer.write().await;
        let back = self.back_buffer.read().await;
        front.copy_from_slice(&back);
    }

    pub async fn present(&self, buffer: Vec<u32>) -> anyhow::Result<()> {
        let mut front = self.front_buffer.write().await;
        front.copy_from_slice(&buffer);
        // In production: ioctl FBIOPUT_VSCREENINFO or DRM page flip
        Ok(())
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
