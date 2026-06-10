use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "kairos-inference-hub",
    version = "1.0.0",
    about = "Speculative inference pipeline orchestrator"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/inference-hub.toml")]
    config: PathBuf,

    #[arg(long)]
    infer: Option<String>,

    #[arg(long)]
    model: Option<String>,

    #[arg(long)]
    metrics: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_inference_hub::config::Config::load(&cli.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();

    info!(
        "kairos-inference-hub v{} starting",
        env!("CARGO_PKG_VERSION")
    );

    let state = kairos_inference_hub::AppState::new(cfg).await?;

    if let Some(prompt) = cli.infer {
        let model = cli.model.as_deref().unwrap_or("default");
        let result = state.infer(model, &prompt).await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if cli.metrics {
        let exporter = kairos_inference_hub::metrics::MetricsExporter::new(state.telemetry.clone());
        println!("{}", serde_json::to_string_pretty(&exporter.collect())?);
        return Ok(());
    }

    let wk = kairos_inference_hub::worker::InferenceWorker::new(
        state.config.clone(),
        state.pipeline.clone(),
        state.scheduler.clone(),
    );
    wk.start().await?;

    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
