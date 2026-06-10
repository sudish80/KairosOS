//! Input manager — touch and keyboard event handling
use crate::config;
use crate::gestures::GestureEngine;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct InputManager {
    config: Arc<RwLock<config::Config>>,
    gesture_engine: Arc<GestureEngine>,
}

pub enum InputEvent {
    Touch(TouchEvent),
    Key(KeyEvent),
    Gesture(super::gestures::Gesture),
}

pub struct TouchEvent {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
}

pub struct KeyEvent {
    pub key: String,
    pub modifiers: Vec<String>,
    pub action: KeyAction,
}

pub enum KeyAction {
    Press,
    Release,
    Repeat,
}

impl InputManager {
    pub fn new(config: Arc<RwLock<config::Config>>, gesture_engine: Arc<GestureEngine>) -> Self {
        Self {
            config,
            gesture_engine,
        }
    }

    pub async fn process_input(&self, event: InputEvent) {
        match event {
            InputEvent::Touch(t) => {
                self.gesture_engine.handle_touch_down(t.x, t.y).await;
            }
            InputEvent::Key(k) => {
                debug!("Key event: {} ({:?})", k.key, k.action);
            }
            InputEvent::Gesture(g) => {
                debug!("Gesture: {:?} at ({}, {})", g.gesture_type, g.x, g.y);
            }
        }
    }

    pub async fn start_input_loop(&self) -> anyhow::Result<()> {
        info!("Input loop started");

        // In production: read from /dev/input/event* and process evdev events
        // For now, simulate occasional input checks
        let gesture_engine = Arc::clone(&self.gesture_engine);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));
            loop {
                interval.tick().await;
                let gestures = gesture_engine.get_pending().await;
                for g in gestures {
                    debug!("Gesture processed: {:?}", g.gesture_type);
                }
            }
        });

        Ok(())
    }
}
