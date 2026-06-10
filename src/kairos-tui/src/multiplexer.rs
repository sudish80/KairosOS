//! TUI multiplexer — manages multiple terminal sessions, tabs, panes
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use crate::config;
use crate::terminal::TerminalEmulator;

pub struct Pane {
    pub id: usize,
    pub title: String,
    pub screen_id: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub active: bool,
}

pub struct Tab {
    pub id: usize,
    pub title: String,
    pub panes: Vec<Pane>,
    pub active_pane: usize,
}

pub struct Multiplexer {
    config: Arc<RwLock<config::Config>>,
    terminal: Arc<TerminalEmulator>,
    tabs: Arc<RwLock<Vec<Tab>>>,
    active_tab: Arc<RwLock<usize>>,
    next_id: Arc<RwLock<usize>>,
}

impl Multiplexer {
    pub fn new(config: Arc<RwLock<config::Config>>, terminal: Arc<TerminalEmulator>) -> Self {
        Self {
            config,
            terminal,
            tabs: Arc::new(RwLock::new(Vec::new())),
            active_tab: Arc::new(RwLock::new(0)),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn create_tab(&self, title: &str) -> usize {
        let mut tabs = self.tabs.write().await;
        let mut next = self.next_id.write().await;
        let id = *next; *next += 1;

        let screen_id = self.terminal.create_screen(80, 24).await;
        let pane = Pane {
            id, title: title.into(), screen_id,
            x: 0, y: 0, width: 80, height: 24, active: true,
        };

        tabs.push(Tab { id, title: title.into(), panes: vec![pane], active_pane: 0 });
        *self.active_tab.write().await = tabs.len() - 1;
        info!("Created tab: {} (id={})", title, id);
        id
    }

    pub async fn split_horizontal(&self, tab_idx: usize) -> anyhow::Result<()> {
        let mut tabs = self.tabs.write().await;
        if tab_idx >= tabs.len() { return Ok(()); }
        let tab = &mut tabs[tab_idx];
        // In production: split pane, create new screen
        Ok(())
    }

    pub async fn write_to_active(&self, data: &str) {
        let tabs = self.tabs.read().await;
        let active = *self.active_tab.read().await;
        if let Some(tab) = tabs.get(active) {
            if let Some(pane) = tab.panes.get(tab.active_pane) {
                self.terminal.write(pane.screen_id, data).await;
            }
        }
    }

    pub async fn switch_tab(&self, idx: usize) {
        let tabs = self.tabs.read().await;
        if idx < tabs.len() {
            *self.active_tab.write().await = idx;
        }
    }

    pub async fn list_tabs(&self) -> Vec<(usize, String)> {
        self.tabs.read().await.iter().map(|t| (t.id, t.title.clone())).collect()
    }

    pub async fn close_tab(&self, idx: usize) {
        let mut tabs = self.tabs.write().await;
        if idx < tabs.len() {
            tabs.remove(idx);
        }
    }
}
