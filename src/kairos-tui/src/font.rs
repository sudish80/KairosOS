//! Font rendering — bitmap glyph atlas, glyph lookup, rasterization
use crate::config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GlyphInfo {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: u32,
    pub bitmap: Vec<u8>,
}

pub struct FontAtlas {
    config: Arc<RwLock<config::Config>>,
    glyphs: HashMap<char, GlyphInfo>,
    line_height: u32,
    baseline: u32,
}

impl FontAtlas {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            glyphs: HashMap::new(),
            line_height: 20,
            baseline: 16,
        }
    }

    pub fn get_glyph(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&ch)
    }

    pub fn render_glyph(&self, ch: char, fg: u32, bg: u32) -> Vec<u32> {
        let w = 8u32;
        let h = 16u32;
        let mut pixels = vec![bg; (w * h) as usize];

        if let Some(glyph) = self.glyphs.get(&ch) {
            for y in 0..glyph.height.min(h) {
                for x in 0..glyph.width.min(w) {
                    let idx = (y * glyph.width + x) as usize;
                    if idx < glyph.bitmap.len() && glyph.bitmap[idx] > 0 {
                        let px = (y * w + x) as usize;
                        if px < pixels.len() {
                            pixels[px] = fg;
                        }
                    }
                }
            }
        }
        pixels
    }

    pub fn text_width(&self, text: &str) -> u32 {
        text.chars().count() as u32 * 8
    }

    pub fn text_height(&self) -> u32 {
        self.line_height
    }
}
