use crate::config::Config;
use crate::error::McpError;
use crate::telemetry::Telemetry;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;

pub struct ProcessHandler {
    config: Arc<RwLock<Config>>,
    telemetry: Arc<Telemetry>,
}

impl ProcessHandler {
    pub fn new(config: Arc<RwLock<Config>>, telemetry: Arc<Telemetry>) -> Self {
        Self { config, telemetry }
    }

    pub async fn handle_request(&self, req: &serde_json::Value) -> serde_json::Value {
        self.telemetry.record_request();
        let method = req["method"].as_str().unwrap_or("");
        let id = &req["id"];
        let result = match method {
            "list_processes" => self.handle_list().await,
            "signal_process" => self.handle_signal(req).await,
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
        let output = Command::new(&cfg.ps_path)
            .args([
                "axo",
                "pid,ppid,user,%cpu,%mem,rss,stat,args",
                "--no-headers",
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(McpError::Internal(format!(
                "ps exited with {}",
                output.status
            )));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed
                .splitn(8, char::is_whitespace)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() < 7 {
                continue;
            }
            let command = if parts.len() > 7 {
                parts[7..].join(" ")
            } else {
                String::new()
            };
            processes.push(serde_json::json!({
                "pid": parts[0].parse::<u64>().unwrap_or(0),
                "ppid": parts[1].parse::<u64>().unwrap_or(0),
                "user": parts[2],
                "cpu": parts[3].parse::<f64>().unwrap_or(0.0),
                "mem": parts[4].parse::<f64>().unwrap_or(0.0),
                "rss": parts[5].parse::<u64>().unwrap_or(0),
                "state": parts[6],
                "command": command,
            }));
            if processes.len() >= cfg.max_processes_returned {
                break;
            }
        }
        self.telemetry.record_list(processes.len() as u64);
        Ok(serde_json::json!({ "processes": processes, "count": processes.len() }))
    }

    async fn handle_signal(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let cfg = self.config.read().await;
        let pid = req["params"]["pid"]
            .as_u64()
            .ok_or(McpError::InvalidParams("missing pid".into()))?;
        let sig = req["params"]["signal"].as_str().unwrap_or("TERM");
        if !cfg
            .allowed_signals
            .iter()
            .any(|s| s.eq_ignore_ascii_case(sig))
        {
            return Err(McpError::SignalNotAllowed(sig.into()));
        }
        let output = Command::new(&cfg.kill_path)
            .args(["-s", sig, &pid.to_string()])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No such process") {
                return Err(McpError::ProcessNotFound(pid));
            }
            return Err(McpError::Internal(format!("kill failed: {}", stderr)));
        }
        self.telemetry.record_signal();
        Ok(serde_json::json!({ "ok": true, "pid": pid, "signal": sig }))
    }
}
