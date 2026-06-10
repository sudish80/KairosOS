use clap::Parser;
use ollama_mcp_server::AppState;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "kairos-ollama-mcp",
    version = "1.0.0",
    about = "Ollama MCP server — LLM inference via Ollama API over JSON-RPC 2.0"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/mcp/ollama.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("ollama_mcp_server=info".parse()?),
        )
        .init();
    let _cli = Cli::parse();
    info!("kairos-ollama-mcp v{} starting", env!("CARGO_PKG_VERSION"));
    let state = AppState::new(ollama_mcp_server::config::Config::default());
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
    info!("kairos-ollama-mcp shutting down");
    Ok(())
}
