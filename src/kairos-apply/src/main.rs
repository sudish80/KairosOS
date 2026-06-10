use std::path::PathBuf;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing::info;

#[derive(Parser)]
#[command(name = "kairos-apply", version = "1.0.0", about = "Declarative configuration state applier")]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/apply.toml")]
    config: PathBuf,

    #[arg(long)]
    apply: Option<PathBuf>,

    #[arg(long)]
    rollback: Option<String>,

    #[arg(long)]
    list: bool,

    #[arg(long)]
    status: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_apply::config::Config::load(&cli.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();

    info!("kairos-apply v{} starting", env!("CARGO_PKG_VERSION"));

    let state = kairos_apply::AppState::new(cfg).await?;

    if let Some(path) = cli.apply {
        let decl = state.parser.parse(&path)?;
        let gen_id = state.applier.apply(&decl).await?;
        println!("Applied generation: {}", gen_id);
        return Ok(());
    }

    if let Some(target) = cli.rollback {
        let rolled = state.rollback_manager.rollback(Some(&target)).await?;
        println!("Rolled back to: {}", rolled);
        return Ok(());
    }

    if cli.list {
        let history = state.generation_store.list_history().await?;
        for gen in &history {
            println!("{} | {} | {} files | {}", gen.id, gen.created_at, gen.file_count, gen.description);
        }
        return Ok(());
    }

    if cli.status {
        let active = state.generation_store.get_active_id().await?;
        println!("Active generation: {:?}", active);
        println!("Rollback health: {:?}", state.rollback_manager.health_check().await?);
        return Ok(());
    }

    let wk = kairos_apply::worker::ApplyWorker::new(
        state.config.clone(),
        state.parser.clone(),
        state.applier.clone(),
    );
    wk.start().await?;

    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
