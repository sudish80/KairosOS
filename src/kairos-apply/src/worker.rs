//! Background worker — watches for new pending configs, auto-applies with health check
use crate::applier::StateApplier;
use crate::config;
use crate::parser::DeclarativeParser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct ApplyWorker {
    config: Arc<RwLock<config::Config>>,
    parser: Arc<DeclarativeParser>,
    applier: Arc<StateApplier>,
    watch_dir: PathBuf,
}

impl ApplyWorker {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        parser: Arc<DeclarativeParser>,
        applier: Arc<StateApplier>,
    ) -> Self {
        let watch_dir = PathBuf::from("/etc/kairos/apply.d");
        Self {
            config,
            parser,
            applier,
            watch_dir,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("ApplyWorker started, watching {:?}", self.watch_dir);
        fs::create_dir_all(&self.watch_dir).await?;

        let watch_dir = self.watch_dir.clone();
        let parser = Arc::clone(&self.parser);
        let applier = Arc::clone(&self.applier);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                match Self::process_pending(&watch_dir, &parser, &applier).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Processed {} pending configs", count);
                        }
                    }
                    Err(e) => error!("Error processing pending configs: {}", e),
                }
            }
        });

        Ok(())
    }

    async fn process_pending(
        watch_dir: &PathBuf,
        parser: &DeclarativeParser,
        applier: &StateApplier,
    ) -> anyhow::Result<usize> {
        let mut count = 0usize;
        let mut reader = fs::read_dir(watch_dir).await?;
        while let Some(entry) = reader.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |e| {
                matches!(e.to_str(), Some("yaml" | "yml" | "toml" | "json"))
            }) {
                match parser.parse(&path) {
                    Ok(decl_config) => match applier.apply(&decl_config).await {
                        Ok(gen_id) => {
                            info!("Applied {} -> {}", path.display(), gen_id);
                            count += 1;
                            let _ = fs::remove_file(&path).await;
                        }
                        Err(e) => error!("Apply failed for {:?}: {}", path, e),
                    },
                    Err(e) => error!("Parse failed for {:?}: {}", path, e),
                }
            }
        }
        Ok(count)
    }
}
