use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "kairos-tui",
    version = "1.0.0",
    about = "Terminal UI daemon with framebuffer and gesture input"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/tui.toml")]
    config: PathBuf,

    #[arg(long)]
    attach: Option<String>,

    #[arg(long)]
    list_sessions: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_tui::config::Config::load(&cli.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();

    info!("kairos-tui v{} starting", env!("CARGO_PKG_VERSION"));

    let state = kairos_tui::AppState::new(cfg).await?;

    if cli.list_sessions {
        let tabs = state.multiplexer.list_tabs().await;
        for (id, title) in &tabs {
            println!("{}: {}", id, title);
        }
        return Ok(());
    }

    // Open DRM/KMS display
    state.drm_manager.open().await?;
    state
        .drm_manager
        .set_mode(state.fb.dimensions().0, state.fb.dimensions().1, 32)
        .await?;

    // Start render worker
    let render_worker = kairos_tui::worker::TuiWorker::new(
        state.config.clone(),
        state.layout_engine.clone(),
        state.fb.clone(),
        state.telemetry.clone(),
    );
    render_worker.start().await?;

    // Start input loop
    state.input_manager.start_input_loop().await?;

    // Create default terminal tab
    let tab_id = state.multiplexer.create_tab("main").await;
    state
        .multiplexer
        .write_to_active("KairosOS Terminal v1.0\n")
        .await;
    info!("Created default tab id={}", tab_id);

    // Initial render
    if let Err(e) = state.render().await {
        error!("Initial render failed: {}", e);
    }

    info!(
        "TUI daemon running on {}x{}",
        state.fb.dimensions().0,
        state.fb.dimensions().1
    );
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
