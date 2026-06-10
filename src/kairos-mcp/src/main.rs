use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "kairos-mcp",
    version = "1.0.0",
    about = "MCP protocol router, service registry, and WASM plugin runtime"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/mcp.toml")]
    config: PathBuf,

    #[arg(long)]
    metrics: bool,

    #[arg(long)]
    discover_plugins: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_mcp::config::Config::load(&cli.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();

    info!("kairos-mcp v{} starting", env!("CARGO_PKG_VERSION"));

    let app_state = kairos_mcp::AppState::new(cfg).await?;

    app_state
        .server
        .register_method("ping", |_| serde_json::json!("pong"))
        .await;
    app_state
        .server
        .register_method("health", |_| serde_json::json!({"status": "ok"}))
        .await;
    app_state
        .server
        .register_method(
            "list_services",
            |_| serde_json::json!({"services": [], "count": 0}),
        )
        .await;

    app_state
        .server
        .register_method("list_plugins", {
            let engine = app_state.plugin_engine.clone();
            move |_| {
                let plugins = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(engine.get_plugins())
                });
                serde_json::to_value(plugins).unwrap_or(serde_json::json!([]))
            }
        })
        .await;

    if cli.discover_plugins {
        app_state.discover_plugins().await?;
        let plugins = app_state.plugin_engine.get_plugins().await;
        info!("Discovered {} WASM plugins", plugins.len());
    }

    info!("Standard MCP methods registered");
    kairos_mcp::run(app_state).await
}
