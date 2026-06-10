use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use crate::config::Config;
use crate::error::McpError;
use crate::telemetry::Telemetry;

pub struct FilesystemHandler {
    config: Arc<RwLock<Config>>,
    telemetry: Arc<Telemetry>,
}

impl FilesystemHandler {
    pub fn new(config: Arc<RwLock<Config>>, telemetry: Arc<Telemetry>) -> Self {
        Self { config, telemetry }
    }

    fn resolve(&self, raw: &str) -> Result<PathBuf, McpError> {
        let path = Path::new(raw).canonicalize().map_err(|_| McpError::PathTraversal(raw.into()))?;
        let cfg = self.config.blocking_read();
        let allowed = cfg.allowed_prefixes.iter().any(|p| path.starts_with(p));
        if !allowed { return Err(McpError::PathTraversal(raw.into())); }
        Ok(path)
    }

    pub async fn handle_request(&self, req: &serde_json::Value) -> serde_json::Value {
        self.telemetry.record_request();
        let method = req["method"].as_str().unwrap_or("");
        let id = &req["id"];
        let result = match method {
            "read_file" => self.handle_read(req).await,
            "write_file" => self.handle_write(req).await,
            "list_dir" => self.handle_list(req).await,
            "stat" => self.handle_stat(req).await,
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

    async fn handle_read(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let path = req["params"]["path"].as_str().ok_or(McpError::InvalidParams("missing path".into()))?;
        let pb = self.resolve(path)?;
        let meta = std::fs::metadata(&pb).map_err(McpError::Io)?;
        let cfg = self.config.read().await;
        if meta.len() > cfg.max_file_size { return Err(McpError::FileTooLarge(meta.len())); }
        let content = tokio::fs::read_to_string(&pb).await?;
        self.telemetry.record_read(meta.len());
        Ok(serde_json::json!({ "content": content, "size": meta.len(), "path": pb.to_string_lossy() }))
    }

    async fn handle_write(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let path = req["params"]["path"].as_str().ok_or(McpError::InvalidParams("missing path".into()))?;
        let content = req["params"]["content"].as_str().ok_or(McpError::InvalidParams("missing content".into()))?;
        let pb = self.resolve(path)?;
        tokio::fs::write(&pb, content).await?;
        self.telemetry.record_write(content.len() as u64);
        Ok(serde_json::json!({ "ok": true, "path": pb.to_string_lossy(), "bytes": content.len() }))
    }

    async fn handle_list(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let path = req["params"]["path"].as_str().unwrap_or(".");
        let pb = self.resolve(path)?;
        let mut entries = Vec::new();
        let mut rd = tokio::fs::read_dir(&pb).await?;
        while let Some(e) = rd.next_entry().await? {
            let mt = e.metadata().await.ok();
            entries.push(serde_json::json!({
                "name": e.file_name().to_string_lossy(),
                "is_dir": mt.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                "is_file": mt.as_ref().map(|m| m.is_file()).unwrap_or(false),
                "size": mt.as_ref().map(|m| m.len()).unwrap_or(0),
            }));
        }
        self.telemetry.record_list();
        Ok(serde_json::json!({ "entries": entries, "path": pb.to_string_lossy() }))
    }

    async fn handle_stat(&self, req: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let path = req["params"]["path"].as_str().ok_or(McpError::InvalidParams("missing path".into()))?;
        let pb = self.resolve(path)?;
        let m = std::fs::metadata(&pb)?;
        let modified = m.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let modified_rfc = chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339();
        self.telemetry.record_stat();
        Ok(serde_json::json!({
            "size": m.len(), "is_dir": m.is_dir(), "is_file": m.is_file(),
            "is_symlink": m.file_type().is_symlink(),
            "modified": modified_rfc,
            "path": pb.to_string_lossy(),
        }))
    }
}
