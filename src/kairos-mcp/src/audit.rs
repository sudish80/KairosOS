//! Audit logging for security and compliance
use std::path::Path;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{error, info};

pub struct AuditLogger {
    config: Arc<tokio::sync::RwLock<crate::config::Config>>,
    log_file: Arc<RwLock<Option<tokio::fs::File>>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub user: Option<String>,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub details: serde_json::Value,
}

impl AuditLogger {
    pub async fn new(
        config: Arc<tokio::sync::RwLock<crate::config::Config>>,
    ) -> anyhow::Result<Self> {
        let log_path = "/var/log/kairos/audit.log";
        if let Some(parent) = Path::new(log_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;
        Ok(Self {
            config,
            log_file: Arc::new(tokio::sync::RwLock::new(Some(file))),
        })
    }

    pub async fn log(&self, entry: AuditEntry) -> anyhow::Result<()> {
        let json = serde_json::to_string(&entry)? + "\n";
        let mut file = self.log_file.write().await;
        if let Some(f) = file.as_mut() {
            f.write_all(json.as_bytes()).await?;
            f.flush().await?;
        }
        Ok(())
    }

    pub async fn log_auth(&self, user: &str, action: &str, success: bool) -> anyhow::Result<()> {
        self.log(AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type: "auth".into(),
            user: Some(user.into()),
            action: action.into(),
            resource: "mcp".into(),
            result: if success { "success" } else { "failure" }.into(),
            details: serde_json::json!({}),
        })
        .await
    }

    pub async fn log_api_call(
        &self,
        method: &str,
        user: &str,
        success: bool,
    ) -> anyhow::Result<()> {
        self.log(AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type: "api".into(),
            user: Some(user.into()),
            action: method.into(),
            resource: "mcp".into(),
            result: if success { "success" } else { "failure" }.into(),
            details: serde_json::json!({}),
        })
        .await
    }

    pub async fn log_config_change(
        &self,
        user: &str,
        key: &str,
        old: &str,
        new: &str,
    ) -> anyhow::Result<()> {
        self.log(AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type: "config".into(),
            user: Some(user.into()),
            action: "change".into(),
            resource: key.into(),
            result: "success".into(),
            details: serde_json::json!({"old": old, "new": new}),
        })
        .await
    }
}
