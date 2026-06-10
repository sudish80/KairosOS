//! Inference scheduler — batch-aware, priority-based request scheduling
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, debug};
use crate::config;
use crate::error::InferenceError;

#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub id: String,
    pub model: String,
    pub prompt: String,
    pub priority: Priority,
    pub submitted_at: Instant,
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone)]
pub struct BatchRequest {
    pub model: String,
    pub requests: Vec<InferenceRequest>,
    pub created_at: Instant,
}

pub struct InferenceScheduler {
    config: Arc<RwLock<config::Config>>,
    queues: Arc<RwLock<BTreeMap<Priority, VecDeque<InferenceRequest>>>>,
    active_batch: Arc<RwLock<Option<BatchRequest>>>,
}

impl InferenceScheduler {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        let mut queues = BTreeMap::new();
        queues.insert(Priority::Low, VecDeque::new());
        queues.insert(Priority::Normal, VecDeque::new());
        queues.insert(Priority::High, VecDeque::new());
        queues.insert(Priority::Critical, VecDeque::new());
        Self {
            config,
            queues: Arc::new(RwLock::new(queues)),
            active_batch: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn enqueue(&self, request: InferenceRequest) {
        let mut queues = self.queues.write().await;
        queues.get_mut(&request.priority)
            .unwrap_or_else(|| {
                let mut q = VecDeque::new();
                q.push_back(request.clone());
                queues.entry(Priority::Normal).or_insert(q)
            })
            .push_back(request);
    }

    pub async fn dequeue_batch(&self) -> Option<BatchRequest> {
        let mut queues = self.queues.write().await;
        let max_batch = self.config.read().await.scheduler.max_batch_size;
        let mut batch_requests = Vec::new();
        let mut model = String::new();

        // Iterate priorities from highest to lowest
        for (_, queue) in queues.iter_mut().rev() {
            while let Some(req) = queue.pop_front() {
                if model.is_empty() {
                    model = req.model.clone();
                }
                if req.model == model && batch_requests.len() < max_batch {
                    batch_requests.push(req);
                } else {
                    // Put back if different model or batch full
                    queue.push_front(req);
                    break;
                }
            }
            if !batch_requests.is_empty() {
                break;
            }
        }

        if batch_requests.is_empty() {
            None
        } else {
            let batch = BatchRequest {
                model,
                requests: batch_requests,
                created_at: Instant::now(),
            };
            *self.active_batch.write().await = Some(batch.clone());
            Some(batch)
        }
    }

    pub async fn queue_depth(&self) -> usize {
        let queues = self.queues.read().await;
        queues.values().map(|q| q.len()).sum()
    }
}
