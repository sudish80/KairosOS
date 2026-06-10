use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use crate::config::Config;
use crate::error::McpError;
use crate::telemetry::Telemetry;

pub struct SystemdHandler {
    config: Arc<RwLock<Config>>,
    telemetry: Arc<Telemetry>,
}

impl SystemdHandler {
    pub fn new(config: Arc<RwLock<Config>>, telemetry: Arc<Telemetry>) -> Self {
        Self { config, telemetry }
    }

    pub async fn handle_request(&self, req: &serde_json::Value) -> serde_json::Value {
        self.telemetry.record_request();
        let method = req["method"].as_str().unwrap_or("");
        let id = &req["id"];
        let result = match method {
            "list_services" => self.handle_list().await,
            "service_status" => self.handle_status(req).await,
            "start_service" | "stop_service" | "restart_service" | "enable_service" | "disable_service" => {
                let action = method.strip_suffix("_service").unwrap_or("restart");
                self.handle_action(req, action).await
            }
            "journal" => self.handle_journal(req).await,
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

    async fn handle_list(&self) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        let output = Command::new(&cfg.systemctl_path)
            .args(["list-units", "--type=service", "--all", "--no-pager", "--no-legend"])
            .output().await?;
        if !output.status.success() {
            return Err(McpError::Internal(format!("systemctl exited with {}", output.status)));
        }
        let stdout = String::from_utf8(output.stdout)?;
        let mut services = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            let parts: Vec<&str> = trimmed.splitn(5, char::is_whitespace).filter(|s| !s.is_empty()).collect();
            if parts.len() < 4 { continue; }
            services.push(serde_json::json!({
                "unit": parts[0],
                "load": parts.get(1).unwrap_or(&""),
                "active": parts.get(2).unwrap_or(&""),
                "sub": parts.get(3).unwrap_or(&""),
                "description": parts.get(4).unwrap_or(&""),
            }));
        }
        self.telemetry.record_list();
        Ok(serde_json::json!({ "services": services, "count": services.len() }))
    }

    async fn handle_status(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        let name = req["params"]["name"].as_str().ok_or(McpError::InvalidParams("missing name".into()))?;
        if name.contains('/') || name.contains(' ') {
            return Err(McpError::InvalidParams("invalid service name".into()));
        }
        let output = Command::new(&cfg.systemctl_path)
            .args(["status", name, "--no-pager", "--lines=20"])
            .output().await?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() && stdout.is_empty() {
            return Err(McpError::ServiceNotFound(name.into()));
        }
        self.telemetry.record_status();
        Ok(serde_json::json!({ "output": stdout, "stderr": stderr, "exit_code": output.status.code().unwrap_or(-1) }))
    }

    async fn handle_action(&self, req: &serde_json::Value, action: &str) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        if !cfg.service_actions.iter().any(|a| a == action) {
            return Err(McpError::ActionNotAllowed(action.into()));
        }
        let name = req["params"]["name"].as_str().ok_or(McpError::InvalidParams("missing name".into()))?;
        if name.contains('/') || name.contains(' ') {
            return Err(McpError::InvalidParams("invalid service name".into()));
        }
        let output = Command::new(&cfg.systemctl_path).args([action, name]).output().await?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            return Err(McpError::ServiceActionFailed(format!("{} {}: {}", action, name, stderr)));
        }
        self.telemetry.record_action();
        Ok(serde_json::json!({ "ok": true, "action": action, "service": name }))
    }

    async fn handle_journal(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        let unit = req["params"]["unit"].as_str().ok_or(McpError::InvalidParams("missing unit".into()))?;
        if unit.contains('/') || unit.contains(' ') {
            return Err(McpError::InvalidParams("invalid unit name".into()));
        }
        let lines = req["params"]["lines"].as_u64().unwrap_or(cfg.default_journal_lines);
        let output = Command::new(&cfg.journalctl_path)
            .args(["-u", unit, "-n", &lines.to_string(), "--no-pager", "-o", "json"])
            .output().await?;
        let stdout = String::from_utf8(output.stdout)?;
        if !output.status.success() {
            return Err(McpError::Internal(format!("journalctl exited with {}", output.status)));
        }
        let entries: Vec<serde_json::Value> = stdout.lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        self.telemetry.record_journal();
        Ok(serde_json::json!({ "entries": entries, "count": entries.len(), "unit": unit }))
    }
}
