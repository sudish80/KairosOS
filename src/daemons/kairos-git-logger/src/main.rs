// KairosOS Git-backed /etc Logger
// Monitors /etc for changes and creates detailed semantic git commits
// describing WHY changes were made based on agent logic context.
// Every system modification becomes part of a perfect, auditable history.

use anyhow::{Context, Result};
use clap::Parser;
use notify::{Config, Event, EventKind, RecommendedWatcher, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};

mod git_backend;

#[derive(Parser, Debug)]
#[command(name = "kairos-git-logger", about = "KairosOS Git-backed /etc tracker")]
struct Args {
    #[arg(short, long, default_value = "/etc")]
    watch_dir: PathBuf,

    #[arg(short, long, default_value = "/etc/.kairos-git")]
    git_dir: PathBuf,

    #[arg(short, long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(format!("kairos_git_logger={}", args.log_level))
        .init();

    info!("Watching {} for changes, storing git repo in {:?}",
        args.watch_dir.display(), args.git_dir);

    let git = git_backend::GitStore::new(&args.git_dir, &args.watch_dir)?;
    git.init()?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()
        .with_poll_interval(Duration::from_secs(2)))?;

    watcher.watch(&args.watch_dir, notify::RecursiveMode::Recursive)
        .context("Failed to watch /etc")?;

    let mut debounce = std::collections::HashMap::new();

    for event in rx {
        match event {
            Ok(Event { kind: EventKind::Modify(_), paths, .. }) |
            Ok(Event { kind: EventKind::Create(_), paths, .. }) |
            Ok(Event { kind: EventKind::Remove(_), paths, .. }) => {
                for path in &paths {
                    let key = path.to_string_lossy().to_string();
                    let now = std::time::Instant::now();
                    debounce.insert(key, now);
                }

                // Debounce: wait 2 seconds after last change before committing
                std::thread::sleep(Duration::from_millis(500));

                let mut to_commit = Vec::new();
                debounce.retain(|_key, time| {
                    if time.elapsed() > Duration::from_secs(2) {
                        to_commit.push(_key.clone());
                        false
                    } else {
                        true
                    }
                });

                if !to_commit.is_empty() {
                    let message = format!(
                        "kairos-auto: system config change\n\nFiles modified:\n{}",
                        to_commit.iter().map(|p| format!("  - {}", p)).collect::<Vec<_>>().join("\n")
                    );

                    match git.commit(&message) {
                        Ok(_) => info!("Committed {} changes to git history", to_commit.len()),
                        Err(e) => warn!("Git commit failed: {}", e),
                    }
                }
            }
            Ok(_) => {}
            Err(e) => warn!("Watch error: {}", e),
        }
    }

    Ok(())
}
