//! KV cache — page-based key-value caching with LRU eviction and compression
use crate::config;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

struct CacheEntry {
    key: String,
    value: String,
    page_id: usize,
    access_count: AtomicU64,
    created_at: std::time::Instant,
}

pub struct KVCache {
    config: Arc<RwLock<config::Config>>,
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    access_order: Arc<RwLock<VecDeque<String>>>,
    page_table: Arc<RwLock<HashMap<usize, Vec<String>>>>,
    current_page: AtomicU64,
    max_entries: usize,
}

impl KVCache {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        let max_entries = config.blocking_read().kv_cache.max_entries;
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(VecDeque::new())),
            page_table: Arc::new(RwLock::new(HashMap::new())),
            current_page: AtomicU64::new(0),
            max_entries,
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut entries = self.entries.write().await;
        let mut access = self.access_order.write().await;

        if let Some(entry) = entries.get(key) {
            let ttl = self.config.read().await.kv_cache.entry_ttl_secs;
            if entry.created_at.elapsed().as_secs() > ttl {
                entries.remove(key);
                access.retain(|k| k != key);
                return None;
            }
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            // Move to front (most recently used)
            access.retain(|k| k != key);
            access.push_front(key.to_string());
            Some(entry.value.clone())
        } else {
            None
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut entries = self.entries.write().await;
        let mut access = self.access_order.write().await;

        if entries.len() >= self.max_entries {
            // Evict LRU
            if let Some(lru_key) = access.pop_back() {
                entries.remove(&lru_key);
            }
        }

        let page_id = self.current_page.fetch_add(1, Ordering::Relaxed) as usize;
        let entry = CacheEntry {
            key: key.clone(),
            value,
            page_id,
            access_count: AtomicU64::new(1),
            created_at: std::time::Instant::now(),
        };

        access.push_front(key.clone());
        entries.insert(key, entry);
    }

    pub async fn clear(&self) {
        self.entries.write().await.clear();
        self.access_order.write().await.clear();
        self.page_table.write().await.clear();
    }

    pub async fn size(&self) -> usize {
        self.entries.read().await.len()
    }

    pub async fn evict_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        let mut access = self.access_order.write().await;
        let ttl = self.config.read().await.kv_cache.entry_ttl_secs;
        let before = entries.len();
        entries.retain(|k, v| {
            let valid = v.created_at.elapsed().as_secs() <= ttl;
            if !valid {
                access.retain(|ak| ak != k);
            }
            valid
        });
        before - entries.len()
    }
}
