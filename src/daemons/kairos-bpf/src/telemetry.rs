// Telemetry data structures and storage
// Stores and indexes events from all 6 eBPF programs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_EVENTS: usize = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub kind: EventKind,
    pub severity: EventSeverity,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    ProcessExec,
    ProcessExit,
    NetworkConnect,
    NetworkBind,
    FileOpen,
    FileWrite,
    FileDelete,
    SyscallAnomaly,
    SchedWakeup,
    SchedSwitch,
    OOMKill,
    OOMScore,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
    Alert,
}

pub struct TelemetryStore {
    events: VecDeque<TelemetryEvent>,
    counters: std::collections::HashMap<String, u64>,
}

impl TelemetryStore {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_EVENTS),
            counters: std::collections::HashMap::new(),
        }
    }

    pub fn push_event(&mut self, event: TelemetryEvent) {
        let source = event.source.clone();
        *self.counters.entry(source).or_insert(0) += 1;

        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn recent_events(&self, count: usize) -> Vec<&TelemetryEvent> {
        self.events.iter().rev().take(count).collect()
    }

    pub fn events_by_source(&self, source: &str, count: usize) -> Vec<&TelemetryEvent> {
        self.events
            .iter()
            .rev()
            .filter(|e| e.source == source)
            .take(count)
            .collect()
    }

    pub fn events_by_severity(&self, severity: EventSeverity, count: usize) -> Vec<&TelemetryEvent> {
        self.events
            .iter()
            .rev()
            .filter(|e| e.severity as u8 >= severity as u8)
            .take(count)
            .collect()
    }

    pub fn counters_summary(&self) -> &std::collections::HashMap<String, u64> {
        &self.counters
    }

    pub fn anomaly_alert(&self) -> Vec<&TelemetryEvent> {
        self.events
            .iter()
            .rev()
            .filter(|e| matches!(e.kind, EventKind::SyscallAnomaly) && matches!(e.severity, EventSeverity::Alert))
            .take(10)
            .collect()
    }
}
