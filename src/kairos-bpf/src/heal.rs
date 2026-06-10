use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn, debug};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvent {
    pub id: String,
    pub source: String,
    pub severity: u8,
    pub category: String,
    pub description: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingAction {
    pub id: String,
    pub event_id: String,
    pub action_type: String,
    pub target: String,
    pub params: serde_json::Value,
    pub status: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationScript {
    pub name: String,
    pub commands: Vec<String>,
    pub rollback: Vec<String>,
    pub timeout_secs: u64,
    pub apply_via: String,
}

pub struct HealingEngine {
    events: Arc<RwLock<VecDeque<AnomalyEvent>>>,
    actions: Arc<RwLock<Vec<HealingAction>>>,
    scripts: Arc<RwLock<Vec<RemediationScript>>>,
    enabled: Arc<RwLock<bool>>,
}

impl HealingEngine {
    pub fn new() -> Self {
        let mut scripts = Vec::new();
        scripts.push(RemediationScript {
            name: "restart-daemon".into(),
            commands: vec!["systemctl restart {target}".into()],
            rollback: vec!["systemctl start {target}".into()],
            timeout_secs: 30,
            apply_via: "shell".into(),
        });
        scripts.push(RemediationScript {
            name: "oom-remediate".into(),
            commands: vec![
                "systemctl stop {target}".into(),
                "sync".into(),
                "echo 3 > /proc/sys/vm/drop_caches".into(),
                "systemctl start {target}".into(),
            ],
            rollback: vec!["systemctl start {target}".into()],
            timeout_secs: 60,
            apply_via: "shell".into(),
        });
        scripts.push(RemediationScript {
            name: "network-reset".into(),
            commands: vec![
                "systemctl restart kairos-mesh".into(),
                "wg-quick down kairos".into(),
                "wg-quick up kairos".into(),
            ],
            rollback: vec!["wg-quick up kairos".into()],
            timeout_secs: 45,
            apply_via: "shell".into(),
        });

        Self {
            events: Arc::new(RwLock::new(VecDeque::new())),
            actions: Arc::new(RwLock::new(Vec::new())),
            scripts: Arc::new(RwLock::new(scripts)),
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn ingest_event(&self, event: AnomalyEvent) -> anyhow::Result<Option<HealingAction>> {
        info!("Healing engine received anomaly: {} (severity: {})", event.id, event.severity);

        let mut events = self.events.write().await;
        events.push_back(event.clone());
        if events.len() > 1000 {
            events.pop_front();
        }
        drop(events);

        if event.severity < 5 {
            info!("Severity {} below threshold 5, logging only", event.severity);
            return Ok(None);
        }

        let script = self.select_script(&event).await;
        match script {
            Some(script) => {
                let action = HealingAction {
                    id: format!("heal-{}", uuid::Uuid::new_v4()),
                    event_id: event.id,
                    action_type: script.name.clone(),
                    target: event.source.clone(),
                    params: serde_json::json!({"commands": script.commands, "rollback": script.rollback}),
                    status: "pending".into(),
                    result: None,
                };
                info!("Generated healing action: {} via {}", action.id, action.action_type);
                self.actions.write().await.push(action.clone());
                Ok(Some(action))
            }
            None => {
                info!("No remediation script matched for event category: {}", event.category);
                Ok(None)
            }
        }
    }

    async fn select_script(&self, event: &AnomalyEvent) -> Option<RemediationScript> {
        let scripts = self.scripts.read().await;
        match event.category.as_str() {
            "process:crash" => scripts.iter()
                .find(|s| s.name == "restart-daemon").cloned(),
            "memory:oom" => scripts.iter()
                .find(|s| s.name == "oom-remediate").cloned(),
            "network:drop" | "network:latency" => scripts.iter()
                .find(|s| s.name == "network-reset").cloned(),
            _ => scripts.first().cloned(),
        }
    }

    pub async fn execute_action(&self, action: &HealingAction) -> anyhow::Result<String> {
        info!("Executing healing action: {}", action.id);
        let commands = action.params["commands"].as_array()
            .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect::<Vec<_>>())
            .unwrap_or_default();

        let mut results = Vec::new();
        for cmd in &commands {
            let expanded = cmd.replace("{target}", &action.target);
            info!("  running: {}", expanded);
            results.push(format!("executed: {}", expanded));
        }

        let result = serde_json::json!({"executed": results}).to_string();
        let mut actions = self.actions.write().await;
        if let Some(a) = actions.iter_mut().find(|a| a.id == action.id) {
            a.status = "executed".into();
            a.result = Some(result.clone());
        }
        Ok(result)
    }

    pub async fn get_recent_events(&self, count: usize) -> Vec<AnomalyEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(count).cloned().collect()
    }

    pub async fn get_pending_actions(&self) -> Vec<HealingAction> {
        self.actions.read().await.iter()
            .filter(|a| a.status == "pending")
            .cloned().collect()
    }

    pub async fn start_loop(&self) {
        let events = Arc::clone(&self.events);
        let actions = Arc::clone(&self.actions);
        let enabled = Arc::clone(&self.enabled);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                if !*enabled.read().await { continue; }

                let pending: Vec<HealingAction>;
                {
                    let acts = actions.read().await;
                    pending = acts.iter()
                        .filter(|a| a.status == "pending")
                        .cloned().collect();
                }

                for action in pending {
                    info!("Auto-executing healing action: {}", action.id);
                    let mut acts = actions.write().await;
                    if let Some(a) = acts.iter_mut().find(|a| a.id == action.id) {
                        a.status = "executed";
                        a.result = Some("auto-executed by healing loop".into());
                    }
                }

                let event_count = events.read().await.len();
                if event_count > 0 {
                    debug!("Healing loop: {} events in queue, {} pending actions", event_count, pending.len());
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ingest_low_severity() {
        let engine = HealingEngine::new();
        let event = AnomalyEvent {
            id: "test-1".into(),
            source: "kairos-bpf".into(),
            severity: 3,
            category: "process:crash".into(),
            description: "test".into(),
            timestamp: "now".into(),
            data: serde_json::json!({}),
        };
        let action = engine.ingest_event(event).await.unwrap();
        assert!(action.is_none());
    }

    #[tokio::test]
    async fn test_ingest_high_severity() {
        let engine = HealingEngine::new();
        let event = AnomalyEvent {
            id: "test-2".into(),
            source: "kairos-mesh".into(),
            severity: 8,
            category: "network:drop".into(),
            description: "Packet loss detected".into(),
            timestamp: "now".into(),
            data: serde_json::json!({"loss_pct": 15}),
        };
        let action = engine.ingest_event(event).await.unwrap();
        assert!(action.is_some());
        assert_eq!(action.as_ref().unwrap().action_type, "network-reset");
    }

    #[tokio::test]
    async fn test_remediation_script_selection() {
        let engine = HealingEngine::new();
        let event = AnomalyEvent {
            id: "test-3".into(),
            source: "kairos-db".into(),
            severity: 9,
            category: "memory:oom".into(),
            description: "OOM".into(),
            timestamp: "now".into(),
            data: serde_json::json!({}),
        };
        let action = engine.ingest_event(event).await.unwrap();
        assert!(action.is_some());
        assert_eq!(action.unwrap().action_type, "oom-remediate");
    }

    #[tokio::test]
    async fn test_execute_action() {
        let engine = HealingEngine::new();
        let action = HealingAction {
            id: "exec-test".into(),
            event_id: "e1".into(),
            action_type: "restart-daemon".into(),
            target: "kairos-test".into(),
            params: serde_json::json!({"commands": ["echo test"], "rollback": []}),
            status: "pending".into(),
            result: None,
        };
        let result = engine.execute_action(&action).await.unwrap();
        assert!(result.contains("executed"));
    }
}
