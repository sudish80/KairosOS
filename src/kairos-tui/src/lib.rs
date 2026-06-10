//! kairos-tui: Terminal UI — framebuffer, DRM/KMS, TUI multiplexer, gesture input
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod telemetry;
pub mod worker;
pub mod framebuffer;
pub mod drm;
pub mod terminal;
pub mod multiplexer;
pub mod gestures;
pub mod input;
pub mod widgets;
pub mod layout;
pub mod colors;
pub mod font;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub fb: Arc<framebuffer::Framebuffer>,
    pub drm_manager: Arc<drm::DrmManager>,
    pub terminal_emulator: Arc<terminal::TerminalEmulator>,
    pub multiplexer: Arc<multiplexer::Multiplexer>,
    pub gesture_engine: Arc<gestures::GestureEngine>,
    pub input_manager: Arc<input::InputManager>,
    pub layout_engine: Arc<layout::LayoutEngine>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new(Arc::clone(&config)));
        let fb = Arc::new(framebuffer::Framebuffer::new(Arc::clone(&config)).await?);
        let drm_manager = Arc::new(drm::DrmManager::new(Arc::clone(&config)).await?);
        let terminal_emulator = Arc::new(terminal::TerminalEmulator::new(Arc::clone(&config)));
        let multiplexer = Arc::new(multiplexer::Multiplexer::new(
            Arc::clone(&config), Arc::clone(&terminal_emulator),
        ));
        let gesture_engine = Arc::new(gestures::GestureEngine::new(Arc::clone(&config)));
        let input_manager = Arc::new(input::InputManager::new(
            Arc::clone(&config), Arc::clone(&gesture_engine),
        ));
        let layout_engine = Arc::new(layout::LayoutEngine::new(Arc::clone(&config), Arc::clone(&multiplexer)));

        info!("kairos-tui AppState initialized");
        Ok(Self { config, telemetry, fb, drm_manager, terminal_emulator, multiplexer, gesture_engine, input_manager, layout_engine })
    }

    pub async fn render(&self) -> anyhow::Result<()> {
        let buffer = self.layout_engine.render().await?;
        self.fb.present(buffer).await?;
        Ok(())
    }
}
