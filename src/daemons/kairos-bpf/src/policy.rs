// Security policy engine for eBPF-based monitoring and enforcement
// Defines rules for what events trigger alerts, blocks, or agent notifications

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    pub source: String,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    CountExceeds { threshold: u64, window_secs: u64 },
    ProcessMatch { pattern: String },
    PathMatch { pattern: String },
    PortMatch { port: u16 },
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    Log,
    Alert,
    Block,
    NotifyAgent,
    CustomScript(String),
}

pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
    counters: HashMap<String, u64>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            counters: HashMap::new(),
        }
    }

    pub fn load_default_policies(&mut self) {
        self.rules = vec![
            PolicyRule {
                name: "ssh-brute-force".into(),
                source: "network".into(),
                condition: PolicyCondition::CountExceeds { threshold: 10, window_secs: 60 },
                action: PolicyAction::Block,
                enabled: true,
            },
            PolicyRule {
                name: "suspicious-exec".into(),
                source: "process".into(),
                condition: PolicyCondition::ProcessMatch { pattern: "/tmp/" }.into(),
                action: PolicyAction::NotifyAgent,
                enabled: true,
            },
            PolicyRule {
                name: "oom-approaching".into(),
                source: "memory".into(),
                condition: PolicyCondition::CountExceeds { threshold: 3, window_secs: 300 },
                action: PolicyAction::NotifyAgent,
                enabled: true,
            },
        ];
    }

    pub fn evaluate(&mut self, source: &str) -> Vec<&PolicyRule> {
        let counter = self.counters.entry(source.to_string()).or_insert(0);
        *counter += 1;

        self.rules.iter()
            .filter(|r| r.enabled && r.source == source)
            .filter(|r| {
                match &r.condition {
                    PolicyCondition::CountExceeds { threshold, .. } => *counter >= *threshold,
                    _ => false,
                }
            })
            .collect()
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|r| r.name != name);
    }

    pub fn list_rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    pub fn reset_counters(&mut self) {
        self.counters.clear();
    }
}
