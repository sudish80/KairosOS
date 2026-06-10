use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTelemetry {
    pub timestamp: f64,
    pub edac_ce_rate: f64,
    pub edac_ue_rate: f64,
    pub tpm_pcr_drift: f64,
    pub prochot_throttle_pct: f64,
    pub bpf_disk_latency_ms: f64,
    pub bpf_oom_rate: f64,
    pub cpu_temperature_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePrediction {
    pub component: String,
    pub probability: f64,
    pub timeframe_hours: f64,
    pub recommended_action: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

pub struct PredictiveAnalyzer {
    history: Arc<RwLock<VecDeque<HardwareTelemetry>>>,
    predictions: Arc<RwLock<Vec<FailurePrediction>>>,
    thresholds: Arc<RwLock<PredictionThresholds>>,
}

#[derive(Debug, Clone)]
pub struct PredictionThresholds {
    pub edac_ue_warn: f64,
    pub edac_ce_warn: f64,
    pub tpm_drift_warn: f64,
    pub prochot_warn: f64,
    pub disk_latency_warn: f64,
    pub oom_warn: f64,
    pub temp_warn: f64,
}

impl Default for PredictionThresholds {
    fn default() -> Self {
        Self {
            edac_ue_warn: 1.0,
            edac_ce_warn: 50.0,
            tpm_drift_warn: 0.1,
            prochot_warn: 30.0,
            disk_latency_warn: 500.0,
            oom_warn: 5.0,
            temp_warn: 85.0,
        }
    }
}

impl PredictiveAnalyzer {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            predictions: Arc::new(RwLock::new(Vec::new())),
            thresholds: Arc::new(RwLock::new(PredictionThresholds::default())),
        }
    }

    pub async fn ingest(&self, telemetry: HardwareTelemetry) {
        let mut history = self.history.write().await;
        history.push_back(telemetry);
        if history.len() > 1000 {
            history.pop_front();
        }
    }

    pub async fn analyze(&self) -> Vec<FailurePrediction> {
        let mut predictions = Vec::new();
        let history = self.history.read().await;
        let thresholds = self.thresholds.read().await;

        if history.len() < 10 {
            return predictions;
        }

        let recent: Vec<_> = history.iter().rev().take(100).cloned().collect();
        let avg = |items: &[f64]| -> f64 { items.iter().sum::<f64>() / items.len() as f64 };
        let trend = |items: &[f64]| -> f64 {
            if items.len() < 2 { return 0.0; }
            let n = items.len() as f64;
            let sum_x: f64 = (0..items.len()).map(|i| i as f64).sum();
            let sum_y: f64 = items.iter().sum();
            let sum_xy: f64 = items.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
            let sum_x2: f64 = (0..items.len()).map(|i| (i as f64) * (i as f64)).sum();
            (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        };

        let edac_ue_vals: Vec<f64> = recent.iter().map(|t| t.edac_ue_rate).collect();
        let edac_ce_vals: Vec<f64> = recent.iter().map(|t| t.edac_ce_rate).collect();
        let tpm_vals: Vec<f64> = recent.iter().map(|t| t.tpm_pcr_drift).collect();
        let temp_vals: Vec<f64> = recent.iter().map(|t| t.cpu_temperature_c).collect();
        let disk_vals: Vec<f64> = recent.iter().map(|t| t.bpf_disk_latency_ms).collect();

        let edac_ue_trend = trend(&edac_ue_vals);
        let edac_ce_trend = trend(&edac_ce_vals);
        let temp_trend = trend(&temp_vals);
        let disk_trend = trend(&disk_vals);

        // Memory failure prediction
        let avg_ue = avg(&edac_ue_vals);
        let avg_ce = avg(&edac_ce_vals);
        if avg_ue > thresholds.edac_ue_warn || (avg_ce > thresholds.edac_ce_warn && edac_ce_trend > 0.0) {
            let prob = (avg_ue * 10.0 + avg_ce / 100.0).min(0.95);
            predictions.push(FailurePrediction {
                component: "memory".into(),
                probability: prob,
                timeframe_hours: if avg_ue > 0.0 { 24.0 } else { 168.0 },
                recommended_action: "Schedule memory replacement. OTA rollback to redundant DIMM configuration.".into(),
                confidence: if prob > 0.7 { "high".into() } else { "medium".into() },
                evidence: vec![
                    format!("UE rate: {:.2}/hr (trend: {:.4})", avg_ue, edac_ue_trend),
                    format!("CE rate: {:.2}/hr (trend: {:.4})", avg_ce, edac_ce_trend),
                ],
            });
        }

        // Thermal failure prediction
        let avg_temp = avg(&temp_vals);
        if avg_temp > thresholds.temp_warn && temp_trend > 0.0 {
            let hours_to_critical = (95.0 - avg_temp) / temp_trend.max(0.1);
            predictions.push(FailurePrediction {
                component: "thermal".into(),
                probability: (avg_temp / 100.0).min(0.9),
                timeframe_hours: hours_to_critical.max(1.0),
                recommended_action: "Increase cooling. Reduce CPU frequency via cpufreq.".into(),
                confidence: if temp_trend > 1.0 { "high".into() } else { "medium".into() },
                evidence: vec![
                    format!("Temperature: {:.1}°C (trend: {:.2}°C/hr)", avg_temp, temp_trend),
                    format!("PROCHOT throttle: {:.1}%", avg(&recent.iter().map(|t| t.prochot_throttle_pct).collect::<Vec<_>>())),
                ],
            });
        }

        // Disk failure prediction
        let avg_disk = avg(&disk_vals);
        if avg_disk > thresholds.disk_latency_warn && disk_trend > 0.0 {
            predictions.push(FailurePrediction {
                component: "storage".into(),
                probability: (avg_disk / 2000.0).min(0.85),
                timeframe_hours: 72.0,
                recommended_action: "Run SMART self-test. Consider proactive disk replacement.".into(),
                confidence: if avg_disk > 1000.0 { "high".into() } else { "medium".into() },
                evidence: vec![
                    format!("Disk latency: {:.1}ms avg (trend: {:.2}ms/hr)", avg_disk, disk_trend),
                ],
            });
        }

        // TPM/PCR integrity
        let avg_tpm_drift = avg(&tpm_vals);
        if avg_tpm_drift > thresholds.tpm_drift_warn {
            predictions.push(FailurePrediction {
                component: "tpm".into(),
                probability: (avg_tpm_drift * 5.0).min(0.8),
                timeframe_hours: 48.0,
                recommended_action: "Re-seal TPM keys. Check for hardware tampering.".into(),
                confidence: "low".into(),
                evidence: vec![
                    format!("PCR drift: {:.4}/hr", avg_tpm_drift),
                ],
            });
        }

        *self.predictions.write().await = predictions.clone();
        predictions
    }

    pub async fn get_predictions(&self) -> Vec<FailurePrediction> {
        self.predictions.read().await.clone()
    }

    pub fn start_analysis_loop(analyzer: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                let predictions = analyzer.analyze().await;
                if !predictions.is_empty() {
                    for p in &predictions {
                        warn!("Failure prediction: {} (prob: {:.2}, timeframe: {:.1}h) — {}",
                            p.component, p.probability, p.timeframe_hours, p.recommended_action);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_predictive_analysis_empty() {
        let analyzer = PredictiveAnalyzer::new();
        let predictions = analyzer.analyze().await;
        assert!(predictions.is_empty());
    }

    #[tokio::test]
    async fn test_memory_failure_prediction() {
        let analyzer = PredictiveAnalyzer::new();
        for i in 0..50 {
            analyzer.ingest(HardwareTelemetry {
                timestamp: i as f64,
                edac_ce_rate: 100.0 + i as f64 * 2.0,
                edac_ue_rate: 0.5 + i as f64 * 0.05,
                tpm_pcr_drift: 0.01,
                prochot_throttle_pct: 5.0,
                bpf_disk_latency_ms: 50.0,
                bpf_oom_rate: 0.1,
                cpu_temperature_c: 60.0,
            }).await;
        }
        let predictions = analyzer.analyze().await;
        let memory = predictions.iter().find(|p| p.component == "memory");
        assert!(memory.is_some());
        assert!(memory.unwrap().probability > 0.0);
    }

    #[tokio::test]
    async fn test_thermal_failure_prediction() {
        let analyzer = PredictiveAnalyzer::new();
        for i in 0..50 {
            analyzer.ingest(HardwareTelemetry {
                timestamp: i as f64,
                edac_ce_rate: 10.0,
                edac_ue_rate: 0.0,
                tpm_pcr_drift: 0.01,
                prochot_throttle_pct: 40.0,
                bpf_disk_latency_ms: 50.0,
                bpf_oom_rate: 0.1,
                cpu_temperature_c: 80.0 + i as f64 * 0.5,
            }).await;
        }
        let predictions = analyzer.analyze().await;
        let thermal = predictions.iter().find(|p| p.component == "thermal");
        assert!(thermal.is_some());
    }

    #[test]
    fn test_threshold_defaults() {
        let t = PredictionThresholds::default();
        assert!(t.edac_ue_warn > 0.0);
        assert!(t.temp_warn > 0.0);
    }
}
