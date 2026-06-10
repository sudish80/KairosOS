use std::sync::Arc; use tokio::sync::RwLock; use crate::config;
pub struct Canvas { config: Arc<RwLock<config::Config>>, width: u32, height: u32, bpp: u32, buffer: Arc<RwLock<Vec<u8>>> }
impl Canvas {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        let cfg = config.blocking_read(); let w = cfg.display.width; let h = cfg.display.height; let b = cfg.display.bpp / 8;
        Self { config, width: w, height: h, bpp: b, buffer: Arc::new(RwLock::new(vec![0u8; (w * h * b) as usize])) }
    }
    pub async fn fill(&self, color: u32) { let buf = &mut *self.buffer.write().await; for pixel in buf.chunks_exact_mut(4) { pixel.copy_from_slice(&color.to_le_bytes()); } }
    pub async fn present(&self) -> anyhow::Result<()> { Ok(()) }
}
