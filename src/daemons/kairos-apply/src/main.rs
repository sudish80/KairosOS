// KairosOS Declarative Config Engine (kairos-apply)
// Reads the declarative configuration, validates it, computes diffs,
// creates atomic generations, and applies changes with rollback support.
// All operations are logged and git-tracked.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::fs;
use tracing::{info, warn, error};

mod parser;
mod validator;
mod generation;
mod rollback;

#[derive(Parser, Debug)]
#[command(name = "kairos-apply", about = "KairosOS Declarative Config Engine")]
enum Cli {
    /// Validate a configuration file
    Validate {
        #[arg(default_value = "/etc/kairos/configuration.nix")]
        config: PathBuf,
    },

    /// Apply a configuration (creates new generation)
    Apply {
        #[arg(default_value = "/etc/kairos/configuration.nix")]
        config: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },

    /// List available generations
    ListGenerations,

    /// Rollback to a previous generation
    Rollback {
        /// Generation ID to rollback to (or "last")
        target: String,
    },

    /// Show current configuration status
    Status,

    /// Diff between current and specified config
    Diff {
        #[arg(default_value = "/etc/kairos/configuration.nix")]
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("kairos_apply=info")
        .init();

    let cli = Cli::parse();

    match cli {
        Cli::Validate { config } => {
            let content = fs::read_to_string(&config)
                .context("Failed to read config file")?;

            match parser::parse_yaml_config(&content) {
                Ok(parsed) => {
                    let issues = validator::validate(&parsed);
                    if issues.is_empty() {
                        info!("Configuration is valid");
                    } else {
                        for issue in &issues {
                            warn!("Validation issue: {}", issue);
                        }
                    }
                }
                Err(e) => {
                    error!("Parse error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Cli::Apply { config, dry_run } => {
            let content = fs::read_to_string(&config)
                .context("Failed to read config file")?;
            let parsed = parser::parse_yaml_config(&content)?;
            let issues = validator::validate(&parsed);

            if !issues.is_empty() {
                error!("Configuration has {} validation issues", issues.len());
                for issue in &issues {
                    error!("  {}", issue);
                }
                std::process::exit(1);
            }

            if dry_run {
                info!("Dry run — would apply configuration");
                generation::preview(&parsed)?;
            } else {
                let gen_id = generation::create(&parsed)?;
                info!("Created generation: {}", gen_id);
                generation::activate(&gen_id)?;
                info!("Configuration applied successfully");
            }
        }

        Cli::ListGenerations => {
            let gens = generation::list()?;
            if gens.is_empty() {
                info!("No generations found");
            } else {
                println!("Generation  Date                        Status");
                println!("-----------  ----                        ------");
                for gen in &gens {
                    println!("{:<12} {}  {}",
                        gen.id,
                        gen.created.format("%Y-%m-%d %H:%M:%S"),
                        if gen.active { "ACTIVE" } else { "" });
                }
            }
        }

        Cli::Rollback { target } => {
            let gen_id = if target == "last" {
                let gens = generation::list()?;
                gens.iter()
                    .filter(|g| !g.active)
                    .last()
                    .map(|g| g.id.clone())
                    .context("No previous generation found")?
            } else {
                target
            };

            let check = rollback::verify(&gen_id)?;
            if !check.can_rollback {
                error!("Cannot rollback to generation {}: {}", gen_id, check.reason);
                std::process::exit(1);
            }

            generation::activate(&gen_id)?;
            info!("Rolled back to generation: {}", gen_id);
        }

        Cli::Status => {
            let gens = generation::list()?;
            let active = gens.iter().find(|g| g.active);
            match active {
                Some(gen) => {
                    println!("Active generation: {} (since {})", gen.id, gen.created);
                    println!("Total generations: {}", gens.len());
                }
                None => {
                    println!("No active generation found");
                }
            }
        }

        Cli::Diff { config } => {
            let content = fs::read_to_string(&config)
                .context("Failed to read config file")?;
            let parsed = parser::parse_yaml_config(&content)?;

            let current = generation::current_config()?
                .unwrap_or_default();

            println!("--- current");
            println!("+++ proposed");
            // Simple diff output
            for (key, val) in &parsed {
                if current.get(key) != Some(val) {
                    println!(" {}:", key);
                    if let Some(old) = current.get(key) {
                        println!("-   {}", old);
                    }
                    println!("+   {}", val);
                }
            }
        }
    }

    Ok(())
}
