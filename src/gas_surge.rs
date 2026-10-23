use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasMetrics {
    pub contract_id: String,
    pub tx_hash: String,
    pub cpu_instructions: u64,
    pub memory_bytes: u64,
    pub ledger_sequence: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSurgeAlert {
    pub contract_id: String,
    pub tx_hash: String,
    pub observed_cpu: u64,
    pub baseline_mean_cpu: f64,
    pub std_deviation: f64,
    pub z_score: f64,
    pub surge_ratio: f64,
    pub ledger_sequence: u32,
    pub severity: String,
}

pub struct GasSurgeDetector {
    window_size: usize,
    history: Vec<u64>,
    z_score_threshold: f64,
    min_samples: usize,
}

impl GasSurgeDetector {
    pub fn new(window_size: usize, z_score_threshold: f64) -> Self {
        Self {
            window_size,
            history: Vec::with_capacity(window_size),
            z_score_threshold,
            min_samples: 5,
        }
    }

    /// Evaluates transaction gas metrics against historical distribution.
    /// Returns a `GasSurgeAlert` if a significant surge is detected.
    pub fn evaluate(&mut self, metric: &GasMetrics) -> Option<GasSurgeAlert> {
        let cpu = metric.cpu_instructions;

        let alert = if self.history.len() >= self.min_samples {
            let mean = self.mean();
            let std_dev = self.std_deviation(mean);

            let z_score = if std_dev > 0.0 {
                (cpu as f64 - mean) / std_dev
            } else {
                0.0
            };
            let surge_ratio = cpu as f64 / mean.max(1.0);

            if (std_dev > 0.0 && z_score >= self.z_score_threshold) || surge_ratio >= 3.0 {
                let severity = if z_score >= 4.0 || surge_ratio >= 5.0 {
                    "CRITICAL_EXPLOIT_RISK".to_string()
                } else {
                    "ELEVATED_SURGE_WARNING".to_string()
                };

                Some(GasSurgeAlert {
                    contract_id: metric.contract_id.clone(),
                    tx_hash: metric.tx_hash.clone(),
                    observed_cpu: cpu,
                    baseline_mean_cpu: mean,
                    std_deviation: std_dev,
                    z_score,
                    surge_ratio,
                    ledger_sequence: metric.ledger_sequence,
                    severity,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Update rolling window
        if self.history.len() >= self.window_size {
            self.history.remove(0);
        }
        self.history.push(cpu);

        alert
    }

    pub fn mean(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.history.iter().sum();
        sum as f64 / self.history.len() as f64
    }

    pub fn std_deviation(&self, mean: f64) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let variance: f64 = self
            .history
            .iter()
            .map(|&val| {
                let diff = val as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / (self.history.len() - 1) as f64;

        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_surge_anomaly_detection() {
        let mut detector = GasSurgeDetector::new(20, 2.5);

        // Baseline transactions around 100k CPU
        for i in 0..10 {
            let m = GasMetrics {
                contract_id: "CDLZ".into(),
                tx_hash: format!("tx_{}", i),
                cpu_instructions: 100_000 + (i as u64 * 100),
                memory_bytes: 1024,
                ledger_sequence: 100 + i as u32,
                timestamp: 1700000000 + i as u64,
            };
            assert!(detector.evaluate(&m).is_none());
        }

        // Sudden 5x surge: 500k CPU instructions
        let surge_tx = GasMetrics {
            contract_id: "CDLZ".into(),
            tx_hash: "tx_surge".into(),
            cpu_instructions: 550_000,
            memory_bytes: 1024,
            ledger_sequence: 111,
            timestamp: 1700000011,
        };

        let alert = detector.evaluate(&surge_tx);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(alert.severity, "CRITICAL_EXPLOIT_RISK");
        assert!(alert.z_score > 2.5);
    }
}
