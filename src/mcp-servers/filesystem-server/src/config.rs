use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub allowed_prefixes: Vec<String>,
    pub max_file_size: u64,
    pub read_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            allowed_prefixes: vec!["/home".into(), "/tmp".into(), "/var/lib/kairos".into()],
            max_file_size: 104_857_600,
            read_buffer_size: 65_536,
        }
    }
}
