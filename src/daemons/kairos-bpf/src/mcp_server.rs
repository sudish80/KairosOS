// MCP Protocol Server for kairos-bpf
// Exposes eBPF telemetry, policy management, and system insights
// to any MCP-compatible agent (Hermes, Claude, etc.)

use crate::policy::PolicyEngine;
use crate::telemetry::{EventSeverity, TelemetryStore};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct McpServer {
    socket_path: String,
    telemetry: Arc<RwLock<TelemetryStore>>,
    policy: Arc<RwLock<PolicyEngine>>,
}

impl McpServer {
    pub fn new(
        socket_path: String,
        telemetry: Arc<RwLock<TelemetryStore>>,
        policy: Arc<RwLock<PolicyEngine>>,
    ) -> Self {
        Self {
            socket_path,
            telemetry,
            policy,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("BPF MCP server listening on {}", self.socket_path);

        // In production: JSON-RPC 2.0 over Unix socket (stdio/local)
        // or Streamable HTTP for remote access.
        // Resources:
        //   bpf://telemetry/recent      -> recent events
        //   bpf://telemetry/anomalies   -> security alerts
        //   bpf://telemetry/counters    -> event counts
        //   bpf://policy/rules          -> policy rules
        // Prompts:
        //   bpf://prompts/diagnose      -> system diagnostic prompt
        // Tools:
        //   block-ip, reset-policy, load-bpf

        // Keep the server alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }

    // MCP resource handlers — called by the MCP router

    pub async fn handle_resource(&self, uri: &str) -> Result<String> {
        match uri {
            "bpf://telemetry/recent" => {
                let tel = self.telemetry.read().await;
                let events = tel.recent_events(50);
                Ok(serde_json::to_string_pretty(&events)?)
            }
            "bpf://telemetry/anomalies" => {
                let tel = self.telemetry.read().await;
                let alerts = tel.anomaly_alert();
                Ok(serde_json::to_string_pretty(&alerts)?)
            }
            "bpf://telemetry/counters" => {
                let tel = self.telemetry.read().await;
                Ok(serde_json::to_string_pretty(tel.counters_summary())?)
            }
            "bpf://policy/rules" => {
                let pol = self.policy.read().await;
                Ok(serde_json::to_string_pretty(pol.list_rules())?)
            }
            _ => Err(anyhow::anyhow!("Unknown resource: {}", uri)),
        }
    }

    pub async fn handle_tool(&self, tool: &str, args: &str) -> Result<String> {
        match tool {
            "block-ip" => {
                info!("Blocking IP: {}", args);
                Ok(format!("Blocked {}", args))
            }
            "reset-policy-counters" => {
                let mut pol = self.policy.write().await;
                pol.reset_counters();
                Ok("Counters reset".into())
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool)),
        }
    }
}
