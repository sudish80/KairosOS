use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDef {
    pub id: String, pub name: String,
    pub command: String, pub dependencies: Vec<String>,
    pub timeout_secs: u64, pub retries: u32,
    pub resources: TaskResources,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResources { pub cpu_cores: f64, pub memory_mb: u64 }
