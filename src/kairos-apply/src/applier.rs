//! State applier — orchestration of parser → validate → diff → store → apply → health-check
use crate::config;
use crate::diff::DiffEngine;
use crate::error::ApplyError;
use crate::generation::GenerationStore;
use crate::parser::{DeclarativeConfig, DeclarativeParser};
use crate::rollback::RollbackManager;
use crate::telemetry::Telemetry;
use crate::validator::ConfigValidator;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct StateApplier {
    config: Arc<RwLock<config::Config>>,
    generation_store: Arc<GenerationStore>,
    validator: Arc<ConfigValidator>,
    diff_engine: Arc<DiffEngine>,
    rollback_manager: Arc<RollbackManager>,
    telemetry: Arc<Telemetry>,
}

impl StateApplier {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        generation_store: Arc<GenerationStore>,
        validator: Arc<ConfigValidator>,
        diff_engine: Arc<DiffEngine>,
        rollback_manager: Arc<RollbackManager>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        Self {
            config,
            generation_store,
            validator,
            diff_engine,
            rollback_manager,
            telemetry,
        }
    }

    pub async fn apply(&self, decl_config: &DeclarativeConfig) -> anyhow::Result<String> {
        let start = Instant::now();

        // 1. Validate
        self.validator
            .validate(decl_config)
            .map_err(|errors| ApplyError::Validation(errors.join("; ")))?;

        // 2. Convert to file specs
        let parser = DeclarativeParser::new(Arc::clone(&self.config));
        let files = parser.to_file_specs(decl_config);

        // 3. Diff against current state
        let current_files = self.load_current_state().await?;
        let diff = self.diff_engine.diff_files(
            &current_files,
            &files.into_iter().collect::<HashMap<_, _>>(),
        );
        info!(
            "Diff: {} added, {} removed, {} modified",
            diff.added.len(),
            diff.removed.len(),
            diff.modified.len()
        );

        // 4. Create generation
        let gen_id = self
            .generation_store
            .create_generation(&decl_config.metadata.description, &[])
            .await?;

        // 5. Apply
        self.generation_store.apply_generation(&gen_id).await?;

        let duration = start.elapsed().as_nanos() as u64;
        self.telemetry.record_apply(true, duration);
        self.telemetry.record_generation_created();

        info!(
            "Apply of generation {} completed in {:?}",
            gen_id,
            start.elapsed()
        );
        Ok(gen_id)
    }

    async fn load_current_state(&self) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        let mut files = HashMap::new();
        let active_link = std::path::Path::new("/etc/kairos/active");

        if !active_link.exists() {
            return Ok(files);
        }

        // Resolve symlink to find active generation directory
        let target = match std::fs::read_link(active_link) {
            Ok(t) => t,
            Err(_) => return Ok(files),
        };

        if !target.exists() || !target.is_dir() {
            return Ok(files);
        }

        // Walk the generation directory tree, reading every file except gen.json
        let mut walk_stack = vec![target.clone()];
        let prefix_len = target.parent().map(|p| p.as_os_str().len()).unwrap_or(0) + 1;

        while let Some(dir) = walk_stack.pop() {
            let mut reader = match tokio::fs::read_dir(&dir).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            while let Some(entry) = reader.next_entry().await? {
                let path = entry.path();
                if path.file_name().map_or(false, |n| n == "gen.json") {
                    continue;
                }
                if entry.file_type().await?.is_dir() {
                    walk_stack.push(path);
                } else {
                    if let Ok(data) = tokio::fs::read(&path).await {
                        // Store relative path from generation root
                        let relative = path
                            .to_string_lossy()
                            .replacen(&target.to_string_lossy().to_string(), "", 1)
                            .trim_start_matches('/')
                            .to_string();
                        files.insert(relative, data);
                    }
                }
            }
        }

        info!(
            "Loaded {} files from active state at {:?}",
            files.len(),
            target
        );
        Ok(files)
    }
}
