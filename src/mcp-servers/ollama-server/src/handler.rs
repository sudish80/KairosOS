use crate::config::Config;
use crate::error::McpError;
use crate::telemetry::Telemetry;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub struct OllamaHandler {
    config: Arc<RwLock<Config>>,
    telemetry: Arc<Telemetry>,
    client: reqwest::Client,
}

impl OllamaHandler {
    pub fn new(config: Arc<RwLock<Config>>, telemetry: Arc<Telemetry>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connection_verbose(true)
            .build()
            .expect("reqwest::Client build");
        Self {
            config,
            telemetry,
            client,
        }
    }

    pub async fn handle_request(&self, req: &serde_json::Value) -> serde_json::Value {
        self.telemetry.record_request();
        let method = req["method"].as_str().unwrap_or("");
        let id = &req["id"];
        let result = match method {
            "generate" => self.handle_generate(req).await,
            "list_models" => self.handle_list_models().await,
            _ => Err(McpError::MethodNotFound(method.into())),
        };
        match result {
            Ok(val) => serde_json::json!({ "jsonrpc": "2.0", "result": val, "id": id }),
            Err(e) => {
                self.telemetry.record_error();
                serde_json::json!({ "jsonrpc": "2.0", "error": { "code": e.code(), "message": e.to_string() }, "id": id })
            }
        }
    }

    async fn handle_generate(
        &self,
        req: &serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        let model = req["params"]["model"]
            .as_str()
            .unwrap_or(&cfg.default_model);
        let prompt = req["params"]["prompt"]
            .as_str()
            .ok_or(McpError::InvalidParams("missing prompt".into()))?;
        let stream = req["params"]["stream"].as_bool().unwrap_or(false);
        let start = Instant::now();
        let resp = self
            .client
            .post(format!("{}/api/generate", cfg.ollama_url))
            .json(&serde_json::json!({ "model": model, "prompt": prompt, "stream": stream }))
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpError::OllamaApi(format!("HTTP {}: {}", status, body)));
        }
        let body: serde_json::Value = resp.json().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        let tokens = body["eval_count"].as_u64().unwrap_or(0);
        self.telemetry.record_generate(tokens, elapsed);
        Ok(body)
    }

    async fn handle_list_models(&self) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        let resp = self
            .client
            .get(format!("{}/api/tags", cfg.ollama_url))
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpError::OllamaApi(format!("HTTP {}: {}", status, body)));
        }
        let body: serde_json::Value = resp.json().await?;
        self.telemetry.record_list_models();
        Ok(body)
    }
}
