use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ollama_url: String,
    pub default_model: String,
    pub request_timeout_secs: u64,
    pub max_history: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            default_model: "hermes-3-llama-3.1:8b-q4_k_m".into(),
            request_timeout_secs: 120,
            max_history: 100,
        }
    }
}
