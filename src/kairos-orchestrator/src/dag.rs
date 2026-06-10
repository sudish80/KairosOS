use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use crate::config;
use crate::task::TaskDef;

#[derive(Debug, Clone)]
pub struct DagNode {
    pub id: String, pub task: TaskDef,
    pub dependencies: Vec<String>, pub dependents: Vec<String>,
    pub status: NodeStatus, pub depth: usize,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeStatus { Pending, Ready, Running, Completed, Failed, Skipped }
pub struct DagEngine { config: Arc<RwLock<config::Config>>, nodes: Arc<RwLock<HashMap<String, DagNode>>> }

impl DagEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config, nodes: Arc::new(RwLock::new(HashMap::new())) } }
    pub async fn build(&self, tasks: Vec<TaskDef>) -> anyhow::Result<Vec<String>> {
        let mut nodes = self.nodes.write().await;
        let mut ids = Vec::new();
        for task in tasks {
            let id = task.id.clone();
            let depth = task.dependencies.iter()
                .filter_map(|d| nodes.get(d)).map(|n| n.depth + 1).max().unwrap_or(0);
            nodes.insert(id.clone(), DagNode {
                id: id.clone(), task, dependencies: Vec::new(), dependents: Vec::new(),
                status: NodeStatus::Pending, depth,
            });
            ids.push(id);
        }
        // Build dependency edges
        let node_ids: Vec<String> = nodes.keys().cloned().collect();
        for id in &node_ids {
            let deps = nodes.get(id).map(|n| n.task.dependencies.clone()).unwrap_or_default();
            for dep in &deps {
                if let Some(parent) = nodes.get_mut(dep) {
                    parent.dependents.push(id.clone());
                }
                if let Some(node) = nodes.get_mut(id) {
                    node.dependencies.push(dep.clone());
                }
            }
        }
        info!("DAG built with {} nodes, max depth {}", ids.len(), ids.iter().filter_map(|id| nodes.get(id)).map(|n| n.depth).max().unwrap_or(0));
        Ok(ids)
    }
    pub async fn get_ready(&self) -> Vec<DagNode> {
        let nodes = self.nodes.read().await;
        nodes.values().filter(|n| n.status == NodeStatus::Pending && n.dependencies.iter().all(|d| nodes.get(d).map_or(false, |p| p.status == NodeStatus::Completed))).cloned().collect()
    }
    pub async fn mark(&self, id: &str, status: NodeStatus) {
        if let Some(node) = self.nodes.write().await.get_mut(id) { node.status = status; }
    }
    pub async fn topological_sort(&self) -> Vec<String> {
        let nodes = self.nodes.read().await;
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for (id, node) in nodes.iter() { in_degree.insert(id.clone(), node.dependencies.len()); }
        let mut queue: VecDeque<String> = in_degree.iter().filter(|(_, &d)| d == 0).map(|(id, _)| id.clone()).collect();
        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() { result.push(id.clone());
            if let Some(node) = nodes.get(&id) { for dep in &node.dependents {
                    if let Some(deg) = in_degree.get_mut(dep) { *deg -= 1; if *deg == 0 { queue.push_back(dep.clone()); } }
                } }
        }
        result
    }
}
