use clap::Parser;
use std::path::PathBuf;
use tracing::info;

static DEFAULT_CONFIG: &str = "/etc/kairos/bpf.toml";
static PID_FILE: &str = "/var/run/kairos/bpf.pid";

#[derive(Parser)]
#[command(
    name = "kairos-bpf",
    version = "1.0.0",
    about = "eBPF telemetry, anomaly detection, and autonomous remediation daemon"
)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_CONFIG)]
    config: PathBuf,
}

fn write_pid() -> anyhow::Result<()> {
    if let Some(parent) = std::path::Path::new(PID_FILE).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(PID_FILE, std::process::id().to_string())?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = if cli.config.exists() {
        kairos_bpf::config::Config::from_file(&cli.config.to_string_lossy())?
    } else {
        info!(
            "Config not found at {}, using defaults",
            cli.config.display()
        );
        kairos_bpf::config::Config::default()
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!(
        "kairos-bpf v{} starting (pid={})",
        env!("CARGO_PKG_VERSION"),
        std::process::id()
    );

    if let Err(e) = write_pid() {
        tracing::warn!("Failed to write PID file: {}", e);
    }

    let state = kairos_bpf::AppState::new(cfg).await?;
    kairos_bpf::run(state).await
}
