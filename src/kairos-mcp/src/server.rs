//! JSON-RPC server — request routing, middleware pipeline, response dispatch
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error, warn};
use crate::protocol::{self, JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};
use crate::config;

type MethodHandler = Arc<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;
type MiddlewareFn = Arc<dyn Fn(&JsonRpcRequest, Option<&JsonRpcResponse>) -> bool + Send + Sync>;

pub struct McpServer {
    config: Arc<RwLock<config::Config>>,
    methods: Arc<RwLock<HashMap<String, MethodHandler>>>,
    middleware: Arc<RwLock<Vec<(String, MiddlewareFn)>>>,
}

impl McpServer {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            methods: Arc::new(RwLock::new(HashMap::new())),
            middleware: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_method<F>(&self, name: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        self.methods.write().await.insert(name.to_string(), Arc::new(handler));
        info!("Registered MCP method: {}", name);
    }

    pub async fn add_middleware<F>(&self, name: &str, f: F)
    where
        F: Fn(&JsonRpcRequest, Option<&JsonRpcResponse>) -> bool + Send + Sync + 'static,
    {
        self.middleware.write().await.push((name.to_string(), Arc::new(f)));
        info!("Added middleware: {}", name);
    }

    pub async fn handle_request(&self, request_str: &str) -> String {
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, protocol::PARSE_ERROR, format!("Parse error: {}", e), None);
                return serde_json::to_string(&resp).unwrap();
            }
        };

        // Run middleware before (check only)
        {
            let middleware = self.middleware.read().await;
            for (name, m) in middleware.iter() {
                if !m(&request, None) {
                    let resp = JsonRpcResponse::error(request.id.clone(), -32000, format!("Blocked by middleware: {}", name), None);
                    return serde_json::to_string(&resp).unwrap();
                }
            }
        }

        // Execute method
        let response = match self.methods.read().await.get(&request.method) {
            Some(handler) => {
                let params = request.params.clone().unwrap_or(serde_json::Value::Null);
                let result = handler(params);
                JsonRpcResponse::success(request.id.clone(), result)
            }
            None => {
                JsonRpcResponse::error(request.id.clone(), protocol::METHOD_NOT_FOUND, format!("Method not found: {}", request.method), None)
            }
        };

        // Run middleware after (logging/audit)
        {
            let middleware = self.middleware.read().await;
            for (_, m) in middleware.iter() {
                m(&request, Some(&response));
            }
        }

        serde_json::to_string(&response).unwrap()
    }

    pub async fn handle_notification(&self, request_str: &str) {
        let notification: JsonRpcNotification = match serde_json::from_str(request_str) {
            Ok(n) => n,
            Err(e) => {
                debug!("Failed to parse notification: {}", e);
                return;
            }
        };

        if let Some(handler) = self.methods.read().await.get(&notification.method) {
            let params = notification.params.unwrap_or(serde_json::Value::Null);
            handler(params);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping_pong() {
        let cfg = Arc::new(RwLock::new(config::Config::default()));
        let server = McpServer::new(cfg);
        server.register_method("ping", |_| serde_json::json!("pong")).await;

        let response = server.handle_request(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).await;
        let resp: JsonRpcResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(resp.result, Some(serde_json::json!("pong")));
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let cfg = Arc::new(RwLock::new(config::Config::default()));
        let server = McpServer::new(cfg);
        let response = server.handle_request(r#"{"jsonrpc":"2.0","id":1,"method":"nonexistent"}"#).await;
        let resp: JsonRpcResponse = serde_json::from_str(&response).unwrap();
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, protocol::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_parse_error() {
        let cfg = Arc::new(RwLock::new(config::Config::default()));
        let server = McpServer::new(cfg);
        let response = server.handle_request("invalid json").await;
        let resp: JsonRpcResponse = serde_json::from_str(&response).unwrap();
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, protocol::PARSE_ERROR);
    }
}
