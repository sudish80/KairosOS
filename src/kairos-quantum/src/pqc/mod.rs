use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use pqcrypto_kyber::kyber768;
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::kem::*;
use pqcrypto_traits::sign::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumAlgorithm {
    Kyber768,
    Dilithium3,
    Hybrid, // Kyber768 + Dilithium3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumConfig {
    pub algorithm: QuantumAlgorithm,
    pub key_ttl_seconds: u64,
    pub session_ttl_seconds: u64,
    pub hybrid_enabled: bool,
}

impl Default for QuantumConfig {
    fn default() -> Self {
        Self {
            algorithm: QuantumAlgorithm::Hybrid,
            key_ttl_seconds: 86400,
            session_ttl_seconds: 3600,
            hybrid_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub algorithm: QuantumAlgorithm,
}

pub struct PostQuantumCrypto {
    config: QuantumConfig,
    key_store: Arc<RwLock<Vec<(String, KeyMaterial, f64)>>>,
}

impl PostQuantumCrypto {
    pub fn new(config: QuantumConfig) -> Self {
        Self {
            config,
            key_store: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn generate_keypair(&self) -> anyhow::Result<KeyMaterial> {
        match self.config.algorithm {
            QuantumAlgorithm::Kyber768 | QuantumAlgorithm::Hybrid => {
                let (pk, sk) = kyber768::keypair();
                Ok(KeyMaterial {
                    public_key: pk.as_bytes().to_vec(),
                    secret_key: sk.as_bytes().to_vec(),
                    algorithm: QuantumAlgorithm::Kyber768,
                })
            }
            QuantumAlgorithm::Dilithium3 => {
                let (pk, sk) = dilithium3::keypair();
                Ok(KeyMaterial {
                    public_key: pk.as_bytes().to_vec(),
                    secret_key: sk.as_bytes().to_vec(),
                    algorithm: QuantumAlgorithm::Dilithium3,
                })
            }
        }
    }

    pub fn encapsulate(&self, peer_public_key: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        match self.config.algorithm {
            QuantumAlgorithm::Kyber768 | QuantumAlgorithm::Hybrid => {
                let pk = kyber768::PublicKey::from_bytes(peer_public_key)
                    .map_err(|e| anyhow::anyhow!("Invalid Kyber768 public key: {:?}", e))?;
                let (shared_secret, ciphertext) = kyber768::encapsulate(&pk);
                Ok((shared_secret.as_bytes().to_vec(), ciphertext.as_bytes().to_vec()))
            }
            QuantumAlgorithm::Dilithium3 => {
                Err(anyhow::anyhow!("Dilithium does not support encapsulation"))
            }
        }
    }

    pub fn decapsulate(&self, ciphertext: &[u8], secret_key: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self.config.algorithm {
            QuantumAlgorithm::Kyber768 | QuantumAlgorithm::Hybrid => {
                let ct = kyber768::Ciphertext::from_bytes(ciphertext)
                    .map_err(|e| anyhow::anyhow!("Invalid Kyber768 ciphertext: {:?}", e))?;
                let sk = kyber768::SecretKey::from_bytes(secret_key)
                    .map_err(|e| anyhow::anyhow!("Invalid Kyber768 secret key: {:?}", e))?;
                let shared_secret = kyber768::decapsulate(&ct, &sk);
                Ok(shared_secret.as_bytes().to_vec())
            }
            QuantumAlgorithm::Dilithium3 => {
                Err(anyhow::anyhow!("Dilithium does not support decapsulation"))
            }
        }
    }

    pub fn sign(&self, message: &[u8], secret_key: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self.config.algorithm {
            QuantumAlgorithm::Dilithium3 | QuantumAlgorithm::Hybrid => {
                let sk = dilithium3::SecretKey::from_bytes(secret_key)
                    .map_err(|e| anyhow::anyhow!("Invalid Dilithium secret key: {:?}", e))?;
                let signature = dilithium3::detached_sign(message, &sk);
                Ok(signature.as_bytes().to_vec())
            }
            QuantumAlgorithm::Kyber768 => {
                Err(anyhow::anyhow!("Kyber768 does not support signing"))
            }
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> anyhow::Result<bool> {
        match self.config.algorithm {
            QuantumAlgorithm::Dilithium3 | QuantumAlgorithm::Hybrid => {
                let pk = dilithium3::PublicKey::from_bytes(public_key)
                    .map_err(|e| anyhow::anyhow!("Invalid Dilithium public key: {:?}", e))?;
                let sig = dilithium3::DetachedSignature::from_bytes(signature)
                    .map_err(|e| anyhow::anyhow!("Invalid Dilithium signature: {:?}", e))?;
                Ok(dilithium3::verify_detached(&sig, message, &pk).is_ok())
            }
            QuantumAlgorithm::Kyber768 => {
                Err(anyhow::anyhow!("Kyber768 does not support verification"))
            }
        }
    }

    pub async fn store_key(&self, name: &str, key: KeyMaterial) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let mut store = self.key_store.write().await;
        store.retain(|(n, _, _)| n != name);
        store.push((name.to_string(), key, now + self.config.key_ttl_seconds as f64));
    }

    pub async fn get_key(&self, name: &str) -> Option<KeyMaterial> {
        let store = self.key_store.read().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        store.iter()
            .find(|(n, _, expiry)| n == name && *expiry > now)
            .map(|(_, key, _)| key.clone())
    }

    pub async fn start_key_rotation(&self) {
        let config = self.config.clone();
        let store = Arc::clone(&self.key_store);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(config.key_ttl_seconds / 2)).await;
                let mut ks = store.write().await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                ks.retain(|(_, _, expiry)| *expiry > now);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_keypair() {
        let pqc = PostQuantumCrypto::new(QuantumConfig {
            algorithm: QuantumAlgorithm::Kyber768,
            ..Default::default()
        });
        let keys = pqc.generate_keypair().unwrap();
        assert!(!keys.public_key.is_empty());
        assert!(!keys.secret_key.is_empty());
    }

    #[test]
    fn test_dilithium_keypair() {
        let pqc = PostQuantumCrypto::new(QuantumConfig {
            algorithm: QuantumAlgorithm::Dilithium3,
            ..Default::default()
        });
        let keys = pqc.generate_keypair().unwrap();
        assert!(!keys.public_key.is_empty());
    }

    #[test]
    fn test_kyber_kem_roundtrip() {
        let alice = PostQuantumCrypto::new(QuantumConfig {
            algorithm: QuantumAlgorithm::Kyber768,
            ..Default::default()
        });
        let bob = PostQuantumCrypto::new(QuantumConfig {
            algorithm: QuantumAlgorithm::Kyber768,
            ..Default::default()
        });

        let bob_keys = bob.generate_keypair().unwrap();
        let (shared_alice, ct) = alice.encapsulate(&bob_keys.public_key).unwrap();
        let shared_bob = bob.decapsulate(&ct, &bob_keys.secret_key).unwrap();

        assert_eq!(shared_alice, shared_bob);
    }

    #[test]
    fn test_dilithium_sign_verify() {
        let pqc = PostQuantumCrypto::new(QuantumConfig {
            algorithm: QuantumAlgorithm::Dilithium3,
            ..Default::default()
        });
        let keys = pqc.generate_keypair().unwrap();
        let msg = b"hello quantum world";
        let sig = pqc.sign(msg, &keys.secret_key).unwrap();
        let valid = pqc.verify(msg, &sig, &keys.public_key).unwrap();
        assert!(valid);
    }

    #[tokio::test]
    async fn test_key_store() {
        let pqc = PostQuantumCrypto::new(QuantumConfig::default());
        let keys = pqc.generate_keypair().unwrap();
        pqc.store_key("node-1", keys.clone()).await;
        let retrieved = pqc.get_key("node-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().public_key, keys.public_key);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let pqc = PostQuantumCrypto::new(QuantumConfig {
            key_ttl_seconds: 1,
            ..Default::default()
        });
        let keys = pqc.generate_keypair().unwrap();
        pqc.store_key("temp", keys).await;
        pqc.start_key_rotation().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let retrieved = pqc.get_key("temp").await;
        assert!(retrieved.is_none());
    }
}
