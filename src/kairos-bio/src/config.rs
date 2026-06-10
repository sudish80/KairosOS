use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub sequence: SeqConfig, pub alignment: AlignConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String, pub data_dir: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeqConfig { pub reference_genome: String, pub max_read_length: u32, pub quality_encoding: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignConfig { pub algorithm: String, pub max_mismatches: u32, pub gap_penalty: i32, pub match_score: i32 }
impl Default for Config { fn default() -> Self { Self {
    general: GeneralConfig { daemonize: true, log_level: "info".into(), data_dir: "/var/lib/kairos/bio".into() },
    sequence: SeqConfig { reference_genome: "/var/lib/kairos/bio/ref/hg38.fasta".into(), max_read_length: 150, quality_encoding: "phred33".into() },
    alignment: AlignConfig { algorithm: "smith-waterman".into(), max_mismatches: 3, gap_penalty: -2, match_score: 1 },
} } }
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
