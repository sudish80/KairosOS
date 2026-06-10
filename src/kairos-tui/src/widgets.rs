//! Widget library — buttons, text inputs, progress bars, menus
use crate::config;
use crate::framebuffer::Framebuffer;

pub struct WidgetRenderer {
    config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
}

impl WidgetRenderer {
    pub fn new(config: std::sync::Arc<tokio::sync::RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub async fn draw_button(
        &self,
        fb: &Framebuffer,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        label: &str,
        active: bool,
    ) {
        let bg = if active { 0x007ACC } else { 0x3C3C3C };
        let fg = 0xD4D4D4;
        fb.draw_rect(x, y, w, h, bg).await;
        fb.draw_text(x + 10, y + h / 4, label, fg, 8, 16).await;
    }

    pub async fn draw_progress_bar(
        &self,
        fb: &Framebuffer,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        progress: f64,
    ) {
        let bg = 0x3C3C3C;
        let fill = 0x4EC9B0;
        fb.draw_rect(x, y, w, h, bg).await;
        let fill_w = (w as f64 * progress.clamp(0.0, 1.0)) as u32;
        if fill_w > 0 {
            fb.draw_rect(x, y, fill_w, h, fill).await;
        }
    }

    pub async fn draw_text_input(
        &self,
        fb: &Framebuffer,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        text: &str,
        focused: bool,
    ) {
        let bg = if focused { 0x2D2D2D } else { 0x1E1E1E };
        let border = if focused { 0x007ACC } else { 0x3C3C3C };
        fb.draw_rect(x, y, w, h, border).await;
        fb.draw_rect(x + 1, y + 1, w - 2, h - 2, bg).await;
        fb.draw_text(x + 4, y + h / 4, text, 0xD4D4D4, 8, 16).await;
    }
}
