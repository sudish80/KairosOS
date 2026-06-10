use std::path::PathBuf;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing::info;

#[derive(Parser)]
#[command(name = "kairos-recovery", version = "1.0.0", about = "A/B partition management and recovery system")]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/recovery.toml")]
    config: PathBuf,

    #[arg(long)]
    update: Option<String>,

    #[arg(long)]
    rollback: bool,

    #[arg(long)]
    health: bool,

    #[arg(long)]
    shell: bool,

    #[arg(long)]
    switch: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_recovery::config::Config::load(&cli.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();

    info!("kairos-recovery v{} starting", env!("CARGO_PKG_VERSION"));

    let state = kairos_recovery::AppState::new(cfg).await?;

    if let Some(image) = cli.update {
        state.update_engine.prepare_update(&image).await?;
        state.update_engine.finalize_update().await?;
        println!("Update prepared and finalized. Reboot to apply.");
        return Ok(());
    }

    if cli.rollback {
        state.update_engine.rollback_update().await?;
        println!("Rollback complete. Reboot to apply.");
        return Ok(());
    }

    if cli.health {
        let report = state.health_checker.check_health().await?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if cli.shell {
        state.recovery_shell.enter_shell("User requested recovery shell").await?;
        return Ok(());
    }

    if cli.switch {
        let inactive = state.partition_manager.get_inactive_slot().await;
        state.boot_manager.switch_slot(&inactive).await?;
        println!("Switched to slot. Reboot to apply.");
        return Ok(());
    }

    let wk = kairos_recovery::worker::RecoveryWorker::new(
        state.config.clone(),
        state.health_checker.clone(),
        state.boot_manager.clone(),
        state.telemetry.clone(),
    );
    wk.start().await?;

    info!("Recovery daemon running");
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
