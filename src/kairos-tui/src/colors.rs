//! Color management — ANSI-to-24-bit conversion, theme parsing
use crate::config;

pub fn parse_hex(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF)
}

pub struct Theme {
    pub foreground: u32,
    pub background: u32,
    pub accent: u32,
    pub error: u32,
    pub success: u32,
    pub warning: u32,
}

impl Theme {
    pub fn from_config(cfg: &config::ColorsConfig) -> Self {
        Self {
            foreground: parse_hex(&cfg.foreground),
            background: parse_hex(&cfg.background),
            accent: parse_hex(&cfg.accent),
            error: parse_hex(&cfg.error),
            success: parse_hex(&cfg.success),
            warning: parse_hex(&cfg.warning),
        }
    }
}

pub fn ansi_to_rgb(code: u8) -> u32 {
    match code {
        0 => 0x000000,
        1 => 0xAA0000,
        2 => 0x00AA00,
        3 => 0xAA5500,
        4 => 0x0000AA,
        5 => 0xAA00AA,
        6 => 0x00AAAA,
        7 => 0xAAAAAA,
        8 => 0x555555,
        9 => 0xFF5555,
        10 => 0x55FF55,
        11 => 0xFFFF55,
        12 => 0x5555FF,
        13 => 0xFF55FF,
        14 => 0x55FFFF,
        15 => 0xFFFFFF,
        _ => 0xFFFFFF,
    }
}
