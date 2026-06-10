#![deny(unsafe_code)]
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = kairos_chaos::ChaosConfig::default();
    let engine = std::sync::Arc::new(kairos_chaos::ChaosEngine::new(cfg));
    engine.start().await;

    tracing::info!("kairos-chaos daemon started with score {}", engine.get_score().await);
    tokio::signal::ctrl_c().await?;
    engine.stop().await;
    tracing::info!("Shutdown signal received");
    Ok(())
}
