use clap::Parser;
use std::path::PathBuf;
use systemd_mcp_server::AppState;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "kairos-systemd-mcp",
    version = "1.0.0",
    about = "Systemd MCP server — manage services and journal via JSON-RPC 2.0"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/mcp/systemd.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("systemd_mcp_server=info".parse()?),
        )
        .init();
    let _cli = Cli::parse();
    info!("kairos-systemd-mcp v{} starting", env!("CARGO_PKG_VERSION"));
    let state = AppState::new(systemd_mcp_server::config::Config::default());
    let mut input_line = String::new();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    use tokio::io::AsyncBufReadExt;
    while stdin.read_line(&mut input_line).await? > 0 {
        if input_line.trim().is_empty() {
            input_line.clear();
            continue;
        }
        let req: serde_json::Value = serde_json::from_str(input_line.trim())?;
        let resp = state.handler.handle_request(&req).await;
        println!("{}", serde_json::to_string(&resp)?);
        input_line.clear();
    }
    info!("kairos-systemd-mcp shutting down");
    Ok(())
}
