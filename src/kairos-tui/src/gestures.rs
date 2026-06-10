//! Gesture recognition — tap, swipe, pinch, long-press detection
use crate::config;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureType {
    Tap,
    DoubleTap,
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    PinchIn,
    PinchOut,
    LongPress,
}

#[derive(Debug, Clone)]
pub struct Gesture {
    pub gesture_type: GestureType,
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub scale: f64,
}

struct TouchPoint {
    x: f64,
    y: f64,
    time: Instant,
    active: bool,
}

pub struct GestureEngine {
    config: Arc<RwLock<config::Config>>,
    touches: Arc<RwLock<Vec<TouchPoint>>>,
    last_tap: Arc<RwLock<Option<(f64, f64, Instant)>>>,
    gesture_queue: Arc<RwLock<Vec<Gesture>>>,
}

impl GestureEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            touches: Arc::new(RwLock::new(Vec::new())),
            last_tap: Arc::new(RwLock::new(None)),
            gesture_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn handle_touch_down(&self, x: f64, y: f64) {
        let mut touches = self.touches.write().await;
        touches.push(TouchPoint {
            x,
            y,
            time: Instant::now(),
            active: true,
        });
    }

    pub async fn handle_touch_move(&self, x: f64, y: f64) {
        let mut touches = self.touches.write().await;
        if let Some(touch) = touches.last_mut() {
            touch.x = x;
            touch.y = y;
        }
    }

    pub async fn handle_touch_up(&self, x: f64, y: f64) -> Option<Gesture> {
        let mut touches = self.touches.write().await;
        if let Some(touch) = touches.pop() {
            let dx = x - touch.x;
            let dy = y - touch.y;
            let duration = touch.time.elapsed();
            let cfg = self.config.read().await;
            let swipe_threshold = cfg.input.swipe_threshold;
            let tap_threshold = cfg.input.tap_threshold;

            // Detect gesture
            if dx.abs() > swipe_threshold || dy.abs() > swipe_threshold {
                let gesture_type = if dx.abs() > dy.abs() {
                    if dx > 0 {
                        GestureType::SwipeRight
                    } else {
                        GestureType::SwipeLeft
                    }
                } else {
                    if dy > 0 {
                        GestureType::SwipeDown
                    } else {
                        GestureType::SwipeUp
                    }
                };
                let gesture = Gesture {
                    gesture_type,
                    x,
                    y,
                    dx,
                    dy,
                    scale: 1.0,
                };
                self.gesture_queue.write().await.push(gesture.clone());
                return Some(gesture);
            }

            if dx.abs() < tap_threshold && dy.abs() < tap_threshold && duration.as_millis() < 300 {
                // Check for double tap
                let mut last_tap = self.last_tap.write().await;
                if let Some((lx, ly, lt)) = *last_tap {
                    if (x - lx).abs() < tap_threshold
                        && (y - ly).abs() < tap_threshold
                        && lt.elapsed().as_millis() < 300
                    {
                        *last_tap = None;
                        let gesture = Gesture {
                            gesture_type: GestureType::DoubleTap,
                            x,
                            y,
                            dx: 0.0,
                            dy: 0.0,
                            scale: 1.0,
                        };
                        return Some(gesture);
                    }
                }
                *last_tap = Some((x, y, Instant::now()));
                let gesture = Gesture {
                    gesture_type: GestureType::Tap,
                    x,
                    y,
                    dx: 0.0,
                    dy: 0.0,
                    scale: 1.0,
                };
                return Some(gesture);
            }
        }
        None
    }

    pub async fn get_pending(&self) -> Vec<Gesture> {
        self.gesture_queue.write().await.drain(..).collect()
    }
}
