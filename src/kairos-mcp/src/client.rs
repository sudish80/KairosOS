//! MCP client — service discovery, connection pooling, request dispatch
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info};

pub struct McpClient {
    registry: Arc<crate::registry::ServiceRegistry>,
    connections: Arc<RwLock<HashMap<String, ConnectionPool>>>,
    semaphore: Arc<Semaphore>,
}

struct ConnectionPool {
    endpoints: Vec<String>,
    current: usize,
    timeout: Duration,
}

impl ConnectionPool {
    fn new(endpoints: Vec<String>, timeout: Duration) -> Self {
        Self {
            endpoints,
            current: 0,
            timeout,
        }
    }

    fn next_endpoint(&mut self) -> Option<&str> {
        if self.endpoints.is_empty() {
            return None;
        }
        let idx = self.current % self.endpoints.len();
        self.current += 1;
        Some(self.endpoints[idx].as_str())
    }
}

impl McpClient {
    pub fn new(registry: Arc<crate::registry::ServiceRegistry>) -> Self {
        Self {
            registry,
            connections: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(64)),
        }
    }

    pub async fn call(
        &self,
        service: &str,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let _permit = self.semaphore.acquire().await?;

        let service_info = self
            .registry
            .resolve(method)
            .await
            .ok_or_else(|| anyhow::anyhow!("No service found for method: {}", method))?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": method,
            "params": params,
        });

        let response = self
            .send_request(&service_info.endpoint, &request.to_string())
            .await?;
        Ok(response)
    }

    async fn send_request(
        &self,
        endpoint: &str,
        request: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let stream =
            tokio::time::timeout(Duration::from_secs(5), UnixStream::connect(endpoint)).await??;

        let (mut reader, mut writer) = stream.into_split();
        writer.write_all(request.as_bytes()).await?;

        let mut buf = vec![0u8; 16384];
        let n = tokio::time::timeout(Duration::from_secs(5), reader.read(&mut buf)).await??;

        let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;
        Ok(response)
    }

    pub async fn notify(
        &self,
        service: &str,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<()> {
        let _permit = self.semaphore.acquire().await?;

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        // Fire-and-forget notification
        tokio::spawn({
            let registry = Arc::clone(&self.registry);
            async move {
                if let Some(info) = registry.resolve(method).await {
                    let _ = tokio::net::UnixStream::connect(&info.endpoint)
                        .await
                        .and_then(|mut s| {
                            async move { s.write_all(notification.to_string().as_bytes()).await }
                                .await
                        });
                }
            }
        });

        Ok(())
    }
}
