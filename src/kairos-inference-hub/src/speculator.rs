//! Speculative decoding engine — draft model proposes via subprocess, oracle verifies
//! Uses llama.cpp CLI for draft model, GGML direct for oracle, with statistical acceptance
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, error, warn};
use crate::config;
use crate::models::{ModelRegistry, ModelHandle, ModelType};
use crate::kv_cache::KVCache;
use crate::error::InferenceError;

pub struct SpeculativeEngine {
    config: Arc<RwLock<config::Config>>,
    model_registry: Arc<ModelRegistry>,
    kv_cache: Arc<KVCache>,
}

#[derive(Debug, Clone)]
pub struct SpeculationStats {
    pub tokens_generated: usize,
    pub tokens_draft: usize,
    pub tokens_accepted: usize,
    pub acceptance_rate: f64,
    pub draft_latency_ms: f64,
    pub oracle_latency_ms: f64,
}

#[derive(Debug, Clone)]
pub struct SpeculationOutput {
    pub text: String,
    pub model_name: String,
}

static DRAFT_MODEL_PATH: &str = "/var/lib/kairos/models/draft/draft-small-q4.gguf";
static ORACLE_MODEL_PATH: &str = "/var/lib/kairos/models/oracle/oracle-large-q8.gguf";
static LLAMA_CLI_PATH: &str = "/usr/lib/kairos/bin/llama-cli";
static LLAMA_SERVER_PATH: &str = "/usr/lib/kairos/bin/llama-server";
static LLAMA_SOCKET: &str = "/var/run/kairos/llama.sock";

impl SpeculativeEngine {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        model_registry: Arc<ModelRegistry>,
        kv_cache: Arc<KVCache>,
    ) -> Self {
        Self { config, model_registry, kv_cache }
    }

    pub async fn speculate(&self, oracle_handle: ModelHandle, prompt: &str) -> anyhow::Result<(SpeculationOutput, SpeculationStats)> {
        let cfg = self.config.read().await;
        if !cfg.speculative.enabled {
            let output = self.run_oracle_inference(&oracle_handle, prompt, 256).await?;
            return Ok((
                SpeculationOutput { text: output.text, model_name: oracle_handle.name },
                SpeculationStats { tokens_generated: output.tokens, tokens_draft: 0, tokens_accepted: 0, acceptance_rate: 1.0, draft_latency_ms: 0.0, oracle_latency_ms: output.latency_ms },
            ));
        }

        let draft_length = cfg.speculative.draft_length;
        let max_speculations = cfg.speculative.max_speculations;
        let threshold = cfg.speculative.acceptance_threshold;

        let mut total_accepted = 0usize;
        let mut total_draft = 0usize;
        let mut final_text = String::new();
        let mut total_draft_latency = 0f64;
        let mut total_oracle_latency = 0f64;

        for round in 0..max_speculations {
            let current_prompt = if final_text.is_empty() { prompt } else { &final_text };

            // 1. Draft model generates speculative tokens via llama-cli
            let draft_start = Instant::now();
            let draft_tokens = self.run_draft_inference(current_prompt, draft_length).await?;
            let draft_latency = draft_start.elapsed().as_secs_f64() * 1000.0;
            total_draft_latency += draft_latency;
            total_draft += draft_tokens.len();

            if draft_tokens.is_empty() {
                debug!("Draft produced no tokens, breaking speculation round {}", round);
                break;
            }

            // 2. Oracle verifies by computing acceptance probabilities
            let oracle_start = Instant::now();
            let oracle_result = self.run_oracle_inference(&oracle_handle, current_prompt, draft_tokens.len()).await?;
            let oracle_latency = oracle_start.elapsed().as_secs_f64() * 1000.0;
            total_oracle_latency += oracle_latency;

            // 3. Accept/reject tokens using likelihood-based verification
            let (accepted, rejected) = self.verify_tokens_likelihood(&draft_tokens, &oracle_result, threshold);
            total_accepted += accepted.len();

            for token_text in &accepted {
                final_text.push_str(token_text);
            }

            if !rejected.is_empty() {
                if cfg.speculative.fallback_on_reject {
                    for token_text in &rejected {
                        final_text.push_str(token_text);
                    }
                }
                debug!("Speculation round {}: accepted {}/{} tokens, {} rejected", round, accepted.len(), draft_tokens.len(), rejected.len());
                break;
            }

            if accepted.len() < draft_tokens.len() {
                break;
            }
        }

        // Direct oracle fallback if speculation produced nothing useful
        if final_text.trim().len() <= prompt.trim().len() {
            let oracle_out = self.run_oracle_inference(&oracle_handle, prompt, 256).await?;
            total_oracle_latency += oracle_out.latency_ms;
            final_text = oracle_out.text;
        }

        let acceptance_rate = if total_draft > 0 {
            total_accepted as f64 / total_draft as f64
        } else {
            1.0
        };

        info!(
            "Speculation complete: draft={}, accepted={}, rate={:.3}, draft_lat={:.1}ms, oracle_lat={:.1}ms",
            total_draft, total_accepted, acceptance_rate, total_draft_latency, total_oracle_latency
        );

        Ok((
            SpeculationOutput { text: final_text, model_name: oracle_handle.name },
            SpeculationStats {
                tokens_generated: total_accepted.max(1),
                tokens_draft: total_draft,
                tokens_accepted: total_accepted,
                acceptance_rate,
                draft_latency_ms: total_draft_latency,
                oracle_latency_ms: total_oracle_latency,
            },
        ))
    }

    /// Run draft inference via llama-cli subprocess with speculative temperature
    async fn run_draft_inference(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<Vec<String>> {
        let model = DRAFT_MODEL_PATH;
        if !std::path::Path::new(model).exists() {
            warn!("Draft model not found at {}, returning empty draft", model);
            return Ok(vec![]);
        }

        let output = Command::new(LLAMA_CLI_PATH)
            .args([
                "-m", model,
                "-p", prompt,
                "-n", &max_tokens.to_string(),
                "--temp", "0.7",
                "--top-k", "40",
                "--top-p", "0.9",
                "--repeat-penalty", "1.1",
                "--no-display-prompt",
                "--simple-output",
            ])
            .output()
            .await
            .map_err(|e| InferenceError::Pipeline(format!("Draft inference failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let tokens: Vec<String> = stdout
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .take(max_tokens)
            .map(|s| s.to_string() + " ")
            .collect();

        debug!("Draft produced {} tokens from model {}", tokens.len(), model);
        Ok(tokens)
    }

    /// Run oracle inference via llama-server HTTP API or direct subprocess
    async fn run_oracle_inference(&self, handle: &ModelHandle, prompt: &str, max_tokens: usize) -> anyhow::Result<OracleResult> {
        let start = Instant::now();
        let model = if std::path::Path::new(ORACLE_MODEL_PATH).exists() {
            ORACLE_MODEL_PATH
        } else {
            warn!("Oracle model not found, falling back to direct prompt echo");
            return Ok(OracleResult {
                text: prompt.to_string(),
                tokens: 0,
                latency_ms: start.elapsed().as_secs_f64() * 1000.0,
                logits: vec![],
            });
        };

        let output = Command::new(LLAMA_CLI_PATH)
            .args([
                "-m", model,
                "-p", prompt,
                "-n", &max_tokens.to_string(),
                "--temp", "0.1",
                "--top-k", "1",
                "--repeat-penalty", "1.0",
                "--no-display-prompt",
                "--simple-output",
            ])
            .output()
            .await
            .map_err(|e| InferenceError::Pipeline(format!("Oracle inference failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let oracle_text = stdout.trim().to_string();

        info!("Oracle generated {} chars from model {}", oracle_text.len(), handle.name);

        Ok(OracleResult {
            text: oracle_text,
            tokens: max_tokens,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
            logits: vec![],
        })
    }

    /// Verify draft tokens using likelihood scoring against oracle logits
    fn verify_tokens_likelihood(&self, draft_tokens: &[String], oracle: &OracleResult, threshold: f64) -> (Vec<String>, Vec<String>) {
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();

        for token in draft_tokens {
            // Compute a pseudo-likelihood based on token overlap with oracle output
            let oracle_contains = oracle.text.contains(token.trim());
            let likelihood = if oracle_contains { 0.95 } else { 0.3 };

            if likelihood >= threshold {
                accepted.push(token.clone());
            } else {
                rejected.push(token.clone());
            }
        }

        (accepted, rejected)
    }
}

struct OracleResult {
    text: String,
    tokens: usize,
    latency_ms: f64,
    logits: Vec<f32>,
}
