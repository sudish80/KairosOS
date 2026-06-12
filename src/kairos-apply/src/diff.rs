//! Configuration diff engine — shows changes between generations
use crate::config;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<FileDiff>,
    pub unchanged: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileDiff {
    pub path: String,
    pub change_type: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffLine {
    pub kind: String,
    pub content: String,
}

pub struct DiffEngine {
    config: Arc<RwLock<config::Config>>,
}

impl DiffEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub fn diff_files(
        &self,
        old: &HashMap<String, Vec<u8>>,
        new: &HashMap<String, Vec<u8>>,
    ) -> DiffResult {
        let old_paths: std::collections::HashSet<_> = old.keys().collect();
        let new_paths: std::collections::HashSet<_> = new.keys().collect();

        let added: Vec<String> = new_paths
            .difference(&old_paths)
            .map(|s| (*s).clone())
            .collect();
        let removed: Vec<String> = old_paths
            .difference(&new_paths)
            .map(|s| (*s).clone())
            .collect();
        let common: Vec<_> = old_paths.intersection(&new_paths).copied().collect();

        let mut modified = Vec::new();
        let mut unchanged = 0usize;

        for path in &common {
            let old_content = String::from_utf8_lossy(old.get(*path).unwrap());
            let new_content = String::from_utf8_lossy(new.get(*path).unwrap());
            if old_content != new_content {
                modified.push(self.compute_diff(path, &old_content, &new_content));
            } else {
                unchanged += 1;
            }
        }

        DiffResult {
            added,
            removed,
            modified,
            unchanged,
        }
    }

    fn compute_diff(&self, path: &&String, old: &str, new: &str) -> FileDiff {
        let diff = TextDiff::from_lines(old, new);
        let hunks: Vec<DiffHunk> = diff
            .grouped_ops(3)
            .iter()
            .map(|ops| {
                let first = ops.first().unwrap();
                let last = ops.last().unwrap();
                let mut lines = Vec::new();
                for op in ops {
                    for change in diff.iter_changes(op) {
                        let kind = match change.tag() {
                            ChangeTag::Delete => "delete",
                            ChangeTag::Insert => "insert",
                            ChangeTag::Equal => "equal",
                        };
                        lines.push(DiffLine {
                            kind: kind.to_string(),
                            content: change.value().to_string(),
                        });
                    }
                }
                DiffHunk {
                    old_start: first.old_range().start,
                    old_lines: first.old_range().len(),
                    new_start: first.new_range().start,
                    new_lines: first.new_range().len(),
                    lines,
                }
            })
            .collect();

        FileDiff {
            path: path.to_string(),
            change_type: "modified".into(),
            hunks,
        }
    }
}
