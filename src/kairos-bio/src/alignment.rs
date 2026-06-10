use std::sync::Arc; use std::cmp; use tokio::sync::RwLock; use crate::config;
pub struct SequenceAligner { config: Arc<RwLock<config::Config>> }
impl SequenceAligner {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub fn smith_waterman(&self, seq1: &str, seq2: &str) -> AlignResult {
        let cfg = self.config.blocking_read();
        let match_score = cfg.alignment.match_score;
        let gap_penalty = cfg.alignment.gap_penalty;
        let rows = seq1.len() + 1; let cols = seq2.len() + 1;
        let mut matrix = vec![0i32; rows * cols];
        let mut max_score = 0i32; let mut max_pos = (0usize, 0usize);

        for i in 1..rows { for j in 1..cols {
            let diag = matrix[(i-1)*cols + (j-1)] + if seq1.as_bytes()[i-1] == seq2.as_bytes()[j-1] { match_score } else { gap_penalty };
            let up = matrix[(i-1)*cols + j] + gap_penalty;
            let left = matrix[i*cols + (j-1)] + gap_penalty;
            let val = cmp::max(0, cmp::max(diag, cmp::max(up, left)));
            matrix[i*cols + j] = val;
            if val > max_score { max_score = val; max_pos = (i, j); }
        }}

        AlignResult { score: max_score, aligned_len: cmp::min(seq1.len(), seq2.len()), mismatches: 0 }
    }
}
pub struct AlignResult { pub score: i32, pub aligned_len: usize, pub mismatches: usize }
