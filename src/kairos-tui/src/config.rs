use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub display: DisplayConfig,
    pub terminal: TerminalConfig,
    pub input: InputConfig,
    pub layout: LayoutConfig,
    pub colors: ColorsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
    pub refresh_rate_hz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub framebuffer_device: String,
    pub drm_device: String,
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub double_buffer: bool,
    pub vsync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub font_path: String,
    pub font_size: u32,
    pub scrollback_lines: usize,
    pub cursor_blink_ms: u64,
    pub history_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub touch_device: String,
    pub keyboard_device: String,
    #[allow(dead_code)]
    pub gesture_debounce_ms: u64,
    pub tap_threshold: f64,
    pub swipe_threshold: f64,
    pub pinch_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub main_pane: String,
    pub sidebar_visible: bool,
    pub sidebar_width: u32,
    pub status_bar: bool,
    pub tab_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsConfig {
    pub foreground: String,
    pub background: String,
    pub accent: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig { daemonize: true, log_level: "info".into(), refresh_rate_hz: 60 },
            display: DisplayConfig {
                framebuffer_device: "/dev/fb0".into(),
                drm_device: "/dev/dri/card0".into(),
                width: 1920, height: 1080, bpp: 32, double_buffer: true, vsync: true,
            },
            terminal: TerminalConfig {
                font_path: "/usr/share/fonts/kairos/terminal.psf".into(),
                font_size: 16, scrollback_lines: 10000, cursor_blink_ms: 500, history_size: 50000,
            },
            input: InputConfig {
                touch_device: "/dev/input/event0".into(),
                keyboard_device: "/dev/input/event1".into(),
                gesture_debounce_ms: 50,
                tap_threshold: 10.0, swipe_threshold: 50.0, pinch_threshold: 30.0,
            },
            layout: LayoutConfig {
                main_pane: "full".into(), sidebar_visible: false, sidebar_width: 300,
                status_bar: true, tab_bar: true,
            },
            colors: ColorsConfig {
                foreground: "#D4D4D4".into(), background: "#1E1E1E".into(),
                accent: "#007ACC".into(), error: "#F44747".into(),
                success: "#4EC9B0".into(), warning: "#CCCC00".into(),
                theme: "dark".into(),
            },
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
