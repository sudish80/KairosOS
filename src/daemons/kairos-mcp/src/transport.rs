// MCP Transport Layer
// Unix socket and HTTP transports for the MCP protocol

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, Resource, Tool, ToolCallResult, Prompt};
use crate::registry::ServiceRegistry;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

pub mod unix {
    use super::*;
    use tokio::net::UnixListener;

    pub async fn serve(path: &str, registry: Arc<RwLock<ServiceRegistry>>) -> Result<()> {
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        info!("MCP Unix socket listening on {}", path);

        loop {
            let (stream, _addr) = listener.accept().await?;
            let reg = registry.clone();
            tokio::spawn(async move {
                handle_connection(stream, reg).await;
            });
        }
    }

    async fn handle_connection(
        stream: tokio::net::UnixStream,
        registry: Arc<RwLock<ServiceRegistry>>,
    ) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        while let Ok(n) = buf_reader.read_line(&mut line).await {
            if n == 0 { break; }

            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(req) => handle_request(req, &registry).await,
                Err(e) => {
                    JsonRpcResponse::parse_error(serde_json::json!(null), &e.to_string())
                }
            };

            let resp_str = serde_json::to_string(&response).unwrap_or_default();
            let _ = writer.write_all(resp_str.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;

            line.clear();
        }
    }
}

pub mod http {
    use super::*;
    use hyper::body::Incoming;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Method, Request, Response, StatusCode};
    use hyper_util::rt::TokioIo;
    use http_body_util::Full;
    use bytes::Bytes;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    pub async fn serve(port: u16, registry: Arc<RwLock<ServiceRegistry>>) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;
        info!("MCP HTTP server listening on {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let reg = registry.clone();

            tokio::spawn(async move {
                let svc = service_fn(move |req: Request<Incoming>| {
                    handle_http(req, reg.clone())
                });
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, svc)
                    .await
                {
                    error!("HTTP connection error: {}", e);
                }
            });
        }
    }

    async fn handle_http(
        req: Request<Incoming>,
        registry: Arc<RwLock<ServiceRegistry>>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let (parts, _body) = req.into_parts();

        match (parts.method, parts.uri.path()) {
            (Method::GET, "/health") => {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::from("ok")))
                    .unwrap())
            }
            (Method::GET, "/api/services") => {
                let reg = registry.read().await;
                let svcs = reg.list_services();
                let json = serde_json::to_string_pretty(&svcs).unwrap_or_default();
                Ok(Response::builder()
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap())
            }
            (Method::POST, "/api/mcp") => {
                let reg = registry.read().await;
                let info = serde_json::json!({
                    "serverInfo": {
                        "name": "kairos-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "resources": {},
                        "tools": {},
                        "prompts": {}
                    }
                });
                Ok(Response::builder()
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(serde_json::to_string_pretty(&info).unwrap_or_default())))
                    .unwrap())
            }
            _ => {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("not found")))
                    .unwrap())
            }
        }
    }
}

async fn handle_request(
    req: JsonRpcRequest,
    registry: &Arc<RwLock<ServiceRegistry>>,
) -> JsonRpcResponse {
    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => {
            JsonRpcResponse::success(id, serde_json::json!({
                "protocolVersion": "2026-03-26",
                "serverInfo": {
                    "name": "kairos-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "resources": {},
                    "tools": {},
                    "prompts": {}
                }
            }))
        }

        "resources/list" => {
            let reg = registry.read().await;
            let services = reg.list_services();
            let resources: Vec<Resource> = services.iter().flat_map(|svc| {
                svc.capabilities.iter().filter_map(|cap| {
                    if cap.starts_with("resources:") {
                        let uri = &cap["resources:".len()..];
                        Some(Resource {
                            uri: uri.to_string(),
                            name: uri.to_string(),
                            description: Some(format!("Resource from {}", svc.name)),
                            mime_type: Some("application/json".into()),
                        })
                    } else { None }
                })
            }).collect();

            JsonRpcResponse::success(id, serde_json::json!({ "resources": resources }))
        }

        "tools/list" => {
            let reg = registry.read().await;
            let services = reg.list_services();
            let tools: Vec<Tool> = services.iter().flat_map(|svc| {
                svc.capabilities.iter().filter_map(|cap| {
                    if cap.starts_with("tools:") {
                        let name = &cap["tools:".len()..];
                        Some(Tool {
                            name: name.to_string(),
                            description: format!("Tool from {}", svc.name),
                            input_schema: serde_json::json!({"type": "object"}),
                        })
                    } else { None }
                })
            }).collect();

            JsonRpcResponse::success(id, serde_json::json!({ "tools": tools }))
        }

        "prompts/list" => {
            JsonRpcResponse::success(id, serde_json::json!({ "prompts": [] }))
        }

        "resources/read" => {
            let uri = req.params.get("uri")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            JsonRpcResponse::success(id, serde_json::json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": format!("Resource {} not yet implemented", uri)
                }]
            }))
        }

        "tools/call" => {
            let name = req.params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            JsonRpcResponse::success(id, serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": format!("Tool {} not yet implemented", name)
                }],
                "isError": false
            }))
        }

        _ => {
            JsonRpcResponse::method_not_found(id, &req.method)
        }
    }
}
