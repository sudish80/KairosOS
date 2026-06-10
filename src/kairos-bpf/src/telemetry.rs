//! Telemetry ring buffer store for eBPF events
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};
use crate::error::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecEvent {
    pub pid: u32,
    pub ppid: u32,
    pub comm: String,
    pub filename: String,
    pub timestamp_ns: u64,
    pub cgroup_id: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcpEvent {
    pub pid: u32,
    pub comm: String,
    pub sport: u16,
    pub dport: u16,
    pub saddr: [u8; 4],
    pub daddr: [u8; 4],
    pub state: u8,
    pub timestamp_ns: u64,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEvent {
    pub pid: u32,
    pub comm: String,
    pub path: String,
    pub operation: FileOp,
    pub timestamp_ns: u64,
    pub latency_ns: u64,
    pub cgroup_id: u64,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum FileOp { Open, Read, Write, Close, Unlink, Create }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnomalyEvent {
    pub pid: u32,
    pub comm: String,
    pub anomaly_type: AnomalyType,
    pub score: f64,
    pub threshold: f64,
    pub timestamp_ns: u64,
    pub context: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum AnomalyType { SyscallFreq, MemoryGrowth, CpuSpike, NetworkBurst, FileStorm }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedEvent {
    pub pid: u32,
    pub comm: String,
    pub latency_ns: u64,
    pub cpu: u32,
    pub timestamp_ns: u64,
    pub cgroup_id: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OomEvent {
    pub pid: u32,
    pub comm: String,
    pub oom_score: i16,
    pub victim_pid: u32,
    pub victim_comm: String,
    pub timestamp_ns: u64,
    pub memory_pressure: u64,
}

pub struct TelemetryStore {
    exec_events: Arc<Mutex<VecDeque<ExecEvent>>>,
    tcp_events: Arc<Mutex<VecDeque<TcpEvent>>>,
    file_events: Arc<Mutex<VecDeque<FileEvent>>>,
    anomaly_events: Arc<Mutex<VecDeque<AnomalyEvent>>>,
    sched_events: Arc<Mutex<VecDeque<SchedEvent>>>,
    oom_events: Arc<Mutex<VecDeque<OomEvent>>>,
    tx: mpsc::Sender<TelemetryEvent>,
    rx: Mutex<mpsc::Receiver<TelemetryEvent>>,
    capacity: usize,
}

#[derive(Debug)]
pub enum TelemetryEvent {
    Exec(ExecEvent),
    Tcp(TcpEvent),
    File(FileEvent),
    Anomaly(AnomalyEvent),
    Sched(SchedEvent),
    Oom(OomEvent),
}

impl TelemetryStore {
    pub fn new() -> Result<Self> {
        let capacity = 100_000;
        let (tx, rx) = mpsc::channel(capacity);
        Ok(Self {
            exec_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity / 6))),
            tcp_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity / 6))),
            file_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity / 6))),
            anomaly_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity / 6))),
            sched_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity / 6))),
            oom_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity / 6))),
            tx, rx: Mutex::new(rx), capacity,
        })
    }

    pub async fn push_exec(&self, event: ExecEvent) {
        let mut q = self.exec_events.lock().await;
        if q.len() >= self.capacity / 6 { q.pop_front(); }
        q.push_back(event);
    }

    pub async fn push_tcp(&self, event: TcpEvent) {
        let mut q = self.tcp_events.lock().await;
        if q.len() >= self.capacity / 6 { q.pop_front(); }
        q.push_back(event);
    }

    pub async fn push_file(&self, event: FileEvent) {
        let mut q = self.file_events.lock().await;
        if q.len() >= self.capacity / 6 { q.pop_front(); }
        q.push_back(event);
    }

    pub async fn push_anomaly(&self, event: AnomalyEvent) {
        let mut q = self.anomaly_events.lock().await;
        if q.len() >= self.capacity / 6 { q.pop_front(); }
        q.push_back(event);
    }

    pub async fn push_sched(&self, event: SchedEvent) {
        let mut q = self.sched_events.lock().await;
        if q.len() >= self.capacity / 6 { q.pop_front(); }
        q.push_back(event);
    }

    pub async fn push_oom(&self, event: OomEvent) {
        let mut q = self.oom_events.lock().await;
        if q.len() >= self.capacity / 6 { q.pop_front(); }
        q.push_back(event);
    }

    pub async fn get_recent_exec(&self, limit: usize) -> Vec<ExecEvent> {
        self.exec_events.lock().await.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_recent_tcp(&self, limit: usize) -> Vec<TcpEvent> {
        self.tcp_events.lock().await.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_recent_file(&self, limit: usize) -> Vec<FileEvent> {
        self.file_events.lock().await.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_recent_anomaly(&self, limit: usize) -> Vec<AnomalyEvent> {
        self.anomaly_events.lock().await.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_recent_sched(&self, limit: usize) -> Vec<SchedEvent> {
        self.sched_events.lock().await.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_recent_oom(&self, limit: usize) -> Vec<OomEvent> {
        self.oom_events.lock().await.iter().rev().take(limit).cloned().collect()
    }

    pub fn is_healthy(&self) -> bool {
        self.capacity > 0
    }

    async fn ingestion_loop(&self) {
        let mut rx = self.rx.lock().await;
        while let Some(event) = rx.recv().await {
            match event {
                TelemetryEvent::Exec(e) => self.push_exec(e).await,
                TelemetryEvent::Tcp(e) => self.push_tcp(e).await,
                TelemetryEvent::File(e) => self.push_file(e).await,
                TelemetryEvent::Anomaly(e) => self.push_anomaly(e).await,
                TelemetryEvent::Sched(e) => self.push_sched(e).await,
                TelemetryEvent::Oom(e) => self.push_oom(e).await,
            }
        }
    }
}
