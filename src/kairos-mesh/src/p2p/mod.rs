use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error, debug};
use serde::{Deserialize, Serialize};

const BLOCK_SIZE: u64 = 4 * 1024 * 1024; // 4 MiB per block

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequest {
    pub release_id: String,
    pub block_index: u32,
    pub block_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub release_id: String,
    pub block_index: u32,
    pub data: Vec<u8>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmManifest {
    pub release_id: String,
    pub version: String,
    pub total_blocks: u32,
    pub block_hashes: Vec<String>,
    pub total_size: u64,
}

pub struct P2pSwarm {
    manifest: Arc<RwLock<Option<SwarmManifest>>>,
    local_blocks: Arc<RwLock<HashMap<u32, Vec<u8>>>>,
    peer_availability: Arc<RwLock<HashMap<String, Vec<u32>>>>,
    listen_port: u16,
    active: Arc<RwLock<bool>>,
}

impl P2pSwarm {
    pub fn new(listen_port: u16) -> Self {
        Self {
            manifest: Arc::new(RwLock::new(None)),
            local_blocks: Arc::new(RwLock::new(HashMap::new())),
            peer_availability: Arc::new(RwLock::new(HashMap::new())),
            listen_port,
            active: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn announce(&self, manifest: SwarmManifest) {
        info!("Announcing P2P swarm for release {}", manifest.release_id);
        *self.manifest.write().await = Some(manifest);
    }

    pub async fn start_listener(&self) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:{}", self.listen_port);
        let listener = TcpListener::bind(&addr).await?;
        info!("P2P block server listening on {}", addr);

        let local_blocks = Arc::clone(&self.local_blocks);
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let blocks = Arc::clone(&local_blocks);
                        tokio::spawn(async move {
                            if let Err(e) = handle_peer(stream, blocks).await {
                                debug!("P2P peer error from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => error!("P2P accept error: {}", e),
                }
            }
        });
        Ok(())
    }

    pub async fn request_block(&self, peer_addr: &str, request: &BlockRequest) -> anyhow::Result<Option<BlockResponse>> {
        let mut stream = TcpStream::connect(peer_addr).await?;
        let req_data = serde_json::to_vec(request)?;

        let len_data = (req_data.len() as u32).to_be_bytes();
        stream.write_all(&len_data).await?;
        stream.write_all(&req_data).await?;
        stream.flush().await?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let resp_len = u32::from_be_bytes(len_buf) as usize;

        let mut resp_buf = vec![0u8; resp_len];
        stream.read_exact(&mut resp_buf).await?;

        let response: BlockResponse = serde_json::from_slice(&resp_buf)?;
        Ok(Some(response))
    }

    pub async fn store_block(&self, index: u32, data: Vec<u8>) {
        self.local_blocks.write().await.insert(index, data);
    }

    pub async fn get_blocks_for_peer(&self) -> HashMap<u32, Vec<u8>> {
        self.local_blocks.read().await.clone()
    }

    pub async fn total_blocks(&self) -> u32 {
        let blocks = self.local_blocks.read().await;
        blocks.len() as u32
    }

    pub async fn shutdown(&self) {
        *self.active.write().await = false;
    }
}

async fn handle_peer(mut stream: TcpStream, blocks: Arc<RwLock<HashMap<u32, Vec<u8>>>>) -> anyhow::Result<()> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let req_len = u32::from_be_bytes(len_buf) as usize;

    let mut req_buf = vec![0u8; req_len];
    stream.read_exact(&mut req_buf).await?;

    let request: BlockRequest = serde_json::from_slice(&req_buf)?;
    let block_data = blocks.read().await.get(&request.block_index).cloned();

    let response = match block_data {
        Some(data) => BlockResponse {
            release_id: request.release_id,
            block_index: request.block_index,
            data,
            hash: request.block_hash,
        },
        None => return Ok(()),
    };

    let resp_data = serde_json::to_vec(&response)?;
    let len_data = (resp_data.len() as u32).to_be_bytes();
    stream.write_all(&len_data).await?;
    stream.write_all(&resp_data).await?;
    stream.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_swarm_create() {
        let swarm = P2pSwarm::new(0);
        assert_eq!(swarm.total_blocks().await, 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let swarm = P2pSwarm::new(0);
        swarm.store_block(0, vec![1, 2, 3, 4]).await;
        swarm.store_block(1, vec![5, 6, 7, 8]).await;
        assert_eq!(swarm.total_blocks().await, 2);

        let blocks = swarm.get_blocks_for_peer().await;
        assert_eq!(blocks.get(&0).unwrap(), &vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_manifest_announce() {
        let swarm = P2pSwarm::new(0);
        let manifest = SwarmManifest {
            release_id: "rel-1".into(),
            version: "1.0.1".into(),
            total_blocks: 10,
            block_hashes: vec!["hash1".into(); 10],
            total_size: 41943040,
        };
        swarm.announce(manifest).await;
        assert_eq!(swarm.total_blocks().await, 0);
    }

    #[test]
    fn test_block_request_serialization() {
        let req = BlockRequest {
            release_id: "test".into(),
            block_index: 5,
            block_hash: "abc123".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: BlockRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.block_index, 5);
        assert_eq!(back.release_id, "test");
    }
}
