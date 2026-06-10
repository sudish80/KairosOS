// KairosOS eBPF Telemetry & Control Daemon
// Rust userspace daemon managing 6 eBPF programs for system monitoring,
// security anomaly detection, and performance telemetry.
// Exposes all telemetry via MCP protocol to the Kairos agent.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

mod bpf;
mod telemetry;
mod policy;
mod mcp_server;

#[derive(Parser, Debug)]
#[command(name = "kairos-bpf", about = "KairosOS eBPF Telemetry Daemon")]
struct Args {
    #[arg(short, long, default_value = "/etc/kairos/bpf-programs")]
    bpf_dir: PathBuf,

    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[arg(short, long)]
    mcp_socket: Option<String>,

    #[arg(long)]
    no_bpf: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(format!("kairos_bpf={}", args.log_level))
        .init();

    info!("Starting KairosOS eBPF Daemon v{}", env!("CARGO_PKG_VERSION"));

    let telemetry = Arc::new(RwLock::new(telemetry::TelemetryStore::new()));
    let policy_engine = Arc::new(RwLock::new(policy::PolicyEngine::new()));

    if !args.no_bpf {
        info!("Loading eBPF programs from {:?}", args.bpf_dir);
        let bpf_loader = bpf::BpfLoader::new(&args.bpf_dir);
        bpf_loader.load_all().await.context("Failed to load eBPF programs")?;
        info!("All eBPF programs loaded successfully");

        let telemetry_clone = telemetry.clone();
        tokio::spawn(async move {
            bpf_loader.start_telemetry_stream(telemetry_clone).await;
        });
    } else {
        warn!("Running without eBPF (--no-bpf flag) — telemetry disabled");
    }

    let mcp_socket = args.mcp_socket
        .unwrap_or_else(|| "/run/kairos/bpf-mcp.sock".to_string());

    let mcp_server = mcp_server::McpServer::new(
        mcp_socket,
        telemetry.clone(),
        policy_engine.clone(),
    );

    info!("Starting MCP server on {}", mcp_socket);
    mcp_server.start().await.context("MCP server failed")?;

    Ok(())
}
