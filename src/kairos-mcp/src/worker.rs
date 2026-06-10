use tokio::sync::mpsc;
pub enum Command { Shutdown, ReloadConfig, Process(String) }
pub async fn worker_loop(mut rx: mpsc::Receiver<Command>) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Shutdown => { tracing::info!("worker shutting down"); break; }
            Command::ReloadConfig => { tracing::info!("reloading config"); }
            Command::Process(data) => { tracing::debug!("processing: {data}"); crate::telemetry::inc(); }
        }
    }
}
