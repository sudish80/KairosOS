// KairosOS MCP Protocol Router
// Central message bus connecting all system services to AI agents.
// Implements the Model Context Protocol (MCP) over stdio and HTTP transports.
// All system capabilities (telemetry, config, services, knowledge graph)
// are exposed as MCP resources, prompts, and tools.

use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

mod protocol;
mod registry;
mod transport;

#[derive(Parser, Debug)]
#[command(name = "kairos-mcp", about = "KairosOS MCP Protocol Router")]
struct Args {
    #[arg(short, long, default_value = "/run/kairos/mcp.sock")]
    unix_socket: String,

    #[arg(short, long, default_value_t = 9876)]
    http_port: u16,

    #[arg(long, default_value = "info")]
    log_level: String,

    #[arg(long)]
    enable_http: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(format!("kairos_mcp={}", args.log_level))
        .init();

    info!("KairosOS MCP Router v{} starting", env!("CARGO_PKG_VERSION"));

    let registry = Arc::new(RwLock::new(registry::ServiceRegistry::new()));

    // Register core system services
    registry.write().await.register(registry::ServiceRegistration {
        name: "kairos-bpf".into(),
        transport: format!("unix:///run/kairos/bpf-mcp.sock"),
        capabilities: vec![
            "resources:bpf://telemetry/*".into(),
            "tools:block-ip".into(),
            "tools:reset-policy-counters".into(),
        ],
    });

    registry.write().await.register(registry::ServiceRegistration {
        name: "kairos-pkg".into(),
        transport: format!("unix:///run/kairos/pkg-mcp.sock"),
        capabilities: vec![
            "resources:kg://*".into(),
            "tools:kg-query".into(),
            "tools:kg-insert".into(),
        ],
    });

    registry.write().await.register(registry::ServiceRegistration {
        name: "kairos-apply".into(),
        transport: format!("unix:///run/kairos/apply-mcp.sock"),
        capabilities: vec![
            "resources:config://*".into(),
            "tools:config-apply".into(),
            "tools:config-validate".into(),
            "tools:config-rollback".into(),
        ],
    });

    registry.write().await.register(registry::ServiceRegistration {
        name: "kairos-systemd".into(),
        transport: format!("unix:///run/kairos/systemd-mcp.sock"),
        capabilities: vec![
            "tools:service-start".into(),
            "tools:service-stop".into(),
            "tools:service-status".into(),
            "tools:service-logs".into(),
        ],
    });

    info!("Registered {} services", registry.read().await.list_services().len());

    // Start transports
    let mut handles = vec![];

    // Unix socket transport (primary)
    let reg = registry.clone();
    handles.push(tokio::spawn(async move {
        transport::unix::serve(&args.unix_socket, reg).await
    }));

    // HTTP transport (optional, for remote access)
    if args.enable_http {
        let reg = registry.clone();
        handles.push(tokio::spawn(async move {
            transport::http::serve(args.http_port, reg).await
        }));
    }

    info!("MCP Router ready. Unix: {}, HTTP: {}",
        args.unix_socket,
        if args.enable_http { format!(":{}", args.http_port) } else { "disabled".into() }
    );

    futures::future::join_all(handles).await;
    Ok(())
}
