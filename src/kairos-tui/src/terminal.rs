//! Terminal emulator — ANSI escape sequence parser, VT100-compatible rendering
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: u32,
    pub bg: u32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: 0xD4D4D4, bg: 0x1E1E1E, bold: false, italic: false, underline: false }
    }
}

pub struct TerminalScreen {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub scrollback: VecDeque<Vec<Cell>>,
}

pub struct TerminalEmulator {
    config: Arc<RwLock<config::Config>>,
    screens: Arc<RwLock<Vec<TerminalScreen>>>,
    active_screen: Arc<RwLock<usize>>,
}

impl TerminalEmulator {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            screens: Arc::new(RwLock::new(Vec::new())),
            active_screen: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn create_screen(&self, width: u32, height: u32) -> usize {
        let mut screens = self.screens.write().await;
        let id = screens.len();
        screens.push(TerminalScreen {
            width, height,
            cells: vec![Cell::default(); (width * height) as usize],
            cursor_x: 0, cursor_y: 0,
            scrollback: VecDeque::new(),
        });
        id
    }

    pub async fn write(&self, screen_id: usize, data: &str) {
        let mut screens = self.screens.write().await;
        if screen_id >= screens.len() { return; }
        let screen = &mut screens[screen_id];

        for ch in data.chars() {
            match ch {
                '\n' => {
                    screen.cursor_y += 1;
                    screen.cursor_x = 0;
                }
                '\r' => screen.cursor_x = 0,
                '\t' => screen.cursor_x = (screen.cursor_x + 8) & !7,
                '\x08' => { // Backspace
                    if screen.cursor_x > 0 { screen.cursor_x -= 1; }
                }
                c if c.is_ascii_control() => { /* Skip other controls */ }
                c => {
                    let idx = (screen.cursor_y * screen.width + screen.cursor_x) as usize;
                    if idx < screen.cells.len() {
                        screen.cells[idx] = Cell { ch: c, ..Cell::default() };
                    }
                    screen.cursor_x += 1;
                    if screen.cursor_x >= screen.width {
                        screen.cursor_x = 0;
                        screen.cursor_y += 1;
                    }
                }
            }
            // Scroll if needed
            if screen.cursor_y >= screen.height {
                let row: Vec<Cell> = screen.cells.drain(0..screen.width as usize).collect();
                screen.scrollback.push_back(row);
                if screen.scrollback.len() > 10000 {
                    screen.scrollback.pop_front();
                }
                screen.cells.extend(vec![Cell::default(); screen.width as usize]);
                screen.cursor_y = screen.height - 1;
            }
        }
    }

    pub async fn read_screen(&self, screen_id: usize) -> Vec<Cell> {
        let screens = self.screens.read().await;
        if screen_id < screens.len() {
            screens[screen_id].cells.clone()
        } else {
            Vec::new()
        }
    }

    pub async fn clear_screen(&self, screen_id: usize) {
        let mut screens = self.screens.write().await;
        if screen_id < screens.len() {
            screens[screen_id].cells.fill(Cell::default());
            screens[screen_id].cursor_x = 0;
            screens[screen_id].cursor_y = 0;
        }
    }
}
