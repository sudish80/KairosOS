//! Rate limiting for API protection
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use tracing::warn;

pub struct RateLimiter {
    config: Arc<tokio::sync::RwLock<crate::config::Config>>,
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    capacity: u32,
    tokens: f64,
    last_refill: Instant,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            last_refill: Instant::now(),
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill = Instant::now();
    }
}

impl RateLimiter {
    pub fn new(config: Arc<tokio::sync::RwLock<crate::config::Config>>) -> Self {
        Self {
            config,
            buckets: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_limit(&self, key: &str, tokens: u32) -> bool {
        let mut buckets = self.buckets.write().await;
        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket::new(100, 10.0) // 100 tokens, 10/sec refill
        });
        bucket.try_consume(tokens)
    }

    pub async fn get_remaining(&self, key: &str) -> u32 {
        let buckets = self.buckets.read().await;
        buckets.get(key).map(|b| b.tokens as u32).unwrap_or(0)
    }

    pub async fn reset(&self, key: &str) {
        self.buckets.write().await.remove(key);
    }
}