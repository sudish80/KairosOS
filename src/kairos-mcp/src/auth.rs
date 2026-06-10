//! Authentication and authorization
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct AuthManager {
    config: Arc<tokio::sync::RwLock<crate::config::Config>>,
    tokens: Arc<RwLock<HashMap<String, TokenInfo>>>,
    tls_config: Option<rustls::ServerConfig>,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub capabilities: Vec<String>,
    pub expires_at: Option<std::time::Instant>,
    pub created_at: std::time::Instant,
}

impl AuthManager {
    pub fn new(config: Arc<tokio::sync::RwLock<crate::config::Config>>) -> Self {
        Self {
            config,
            tokens: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            tls_config: None,
        }
    }

    pub async fn verify_token(&self, token: &str) -> Option<TokenInfo> {
        let tokens = self.tokens.read().await;
        tokens.get(token).cloned()
    }

    pub async fn issue_token(&self, capabilities: Vec<String>, ttl_secs: Option<u64>) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        let info = TokenInfo {
            token: token.clone(),
            capabilities,
            expires_at: ttl_secs.map(|t| std::time::Instant::now() + std::time::Duration::from_secs(t)),
            created_at: std::time::Instant::now(),
        };
        self.tokens.write().await.insert(token.clone(), info);
        token
    }

    pub async fn revoke_token(&self, token: &str) -> bool {
        self.tokens.write().await.remove(token).is_some()
    }

    pub async fn cleanup_expired(&self) {
        let mut tokens = self.tokens.write().await;
        let now = std::time::Instant::now();
        tokens.retain(|_, info| {
            info.expires_at.map_or(true, |exp| exp > now)
        });
    }
}