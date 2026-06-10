//! Transport layer — Unix sockets, TCP with full request routing through McpServer
use crate::config;
use crate::server::McpServer;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct TransportManager {
    config: Arc<RwLock<config::Config>>,
    server: Arc<McpServer>,
}

impl TransportManager {
    pub fn new(config: Arc<RwLock<config::Config>>, server: Arc<McpServer>) -> Self {
        Self { config, server }
    }

    pub async fn start_all(&self) -> anyhow::Result<()> {
        let cfg = self.config.read().await;

        // Start Unix socket listener
        let unix_path = &cfg.transport.unix_socket_path;
        // Ensure parent dir exists
        if let Some(parent) = Path::new(unix_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        // Remove stale socket
        let _ = tokio::fs::remove_file(unix_path).await;

        match UnixListener::bind(unix_path) {
            Ok(listener) => {
                info!("MCP Unix socket listening on {}", unix_path);
                // Set permissions to 0700
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(unix_path, std::fs::Permissions::from_mode(0o700))
                        .ok();
                }
                let server = Arc::clone(&self.server);
                let max_conn = cfg.transport.max_connections as usize;
                let semaphore = Arc::new(tokio::sync::Semaphore::new(max_conn));
                tokio::spawn(Self::accept_unix_loop(listener, server, semaphore));
            }
            Err(e) => {
                error!("Failed to bind Unix socket {}: {}", unix_path, e);
            }
        }

        // Start TCP listener
        let tcp_bind = &cfg.transport.tcp_bind;
        match TcpListener::bind(tcp_bind).await {
            Ok(listener) => {
                info!("MCP TCP listening on {}", tcp_bind);
                let server2 = Arc::clone(&self.server);
                let max_conn2 = cfg.transport.max_connections as usize;
                let semaphore2 = Arc::new(tokio::sync::Semaphore::new(max_conn2));
                tokio::spawn(Self::accept_tcp_loop(listener, server2, semaphore2));
            }
            Err(e) => {
                error!("Failed to bind TCP {}: {}", tcp_bind, e);
            }
        }

        Ok(())
    }

    async fn accept_unix_loop(
        listener: UnixListener,
        server: Arc<McpServer>,
        semaphore: Arc<tokio::sync::Semaphore>,
    ) {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let permit = semaphore.clone().acquire_owned().await;
                    let server = Arc::clone(&server);
                    tokio::spawn(async move {
                        let _permit = permit;
                        if let Err(e) = Self::handle_client(stream, &server, "unix").await {
                            debug!("Unix client {} disconnected: {}", format!("{:?}", addr), e);
                        }
                    });
                }
                Err(e) => {
                    error!("Unix accept error: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn accept_tcp_loop(
        listener: TcpListener,
        server: Arc<McpServer>,
        semaphore: Arc<tokio::sync::Semaphore>,
    ) {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let permit = semaphore.clone().acquire_owned().await;
                    let server = Arc::clone(&server);
                    tokio::spawn(async move {
                        let _permit = permit;
                        info!("TCP connection from {}", addr);
                        if let Err(e) = Self::handle_client(stream, &server, "tcp").await {
                            debug!("TCP client {} disconnected: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("TCP accept error: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn handle_client(
        mut stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
        server: &McpServer,
        transport: &str,
    ) -> anyhow::Result<()> {
        let mut buf = vec![0u8; 65536];
        let timeout = std::time::Duration::from_secs(30);

        loop {
            let read_fut = stream.read(&mut buf);
            match tokio::time::timeout(timeout, read_fut).await {
                Ok(Ok(0)) => break, // EOF
                Ok(Ok(n)) => {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    debug!("{} request ({} bytes)", transport, n);

                    // Route through McpServer
                    let response = server.handle_request(&request).await;

                    // Write response with newline delimiter
                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        error!("Write error on {}: {}", transport, e);
                        break;
                    }
                    if let Err(e) = stream.flush().await {
                        error!("Flush error on {}: {}", transport, e);
                        break;
                    }
                }
                Ok(Err(e)) => {
                    if e.kind() != std::io::ErrorKind::ConnectionReset {
                        error!("Read error on {}: {}", transport, e);
                    }
                    break;
                }
                Err(_) => {
                    // Timeout — send keepalive or disconnect
                    debug!("{} client timeout, disconnecting", transport);
                    break;
                }
            }
        }
        Ok(())
    }
}
