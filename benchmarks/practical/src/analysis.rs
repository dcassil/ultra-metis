use crate::types::{BenchmarkRun, GateDecision, RunMetrics};

pub struct BenchmarkAnalysis {
    autonomous_run: BenchmarkRun,
    validated_run: BenchmarkRun,
}

impl BenchmarkAnalysis {
    pub fn new(autonomous_run: BenchmarkRun, validated_run: BenchmarkRun) -> Self {
        Self {
            autonomous_run,
            validated_run,
        }
    }

    /// Compare metrics between autonomous and validated runs
    pub fn compare(&self) -> ComparisonReport {
        let token_overhead = calculate_token_overhead(
            &self.autonomous_run.total_metrics,
            &self.validated_run.total_metrics,
        );

        let quality_delta = calculate_quality_delta(
            &self.autonomous_run.total_metrics,
            &self.validated_run.total_metrics,
        );

        let roi = calculate_roi(token_overhead, quality_delta);

        ComparisonReport {
            token_overhead,
            quality_delta,
            roi,
            error_detection_rate: self.calculate_error_detection_rate(),
            gate_effectiveness: self
                .validated_run
                .total_metrics
                .gate_effectiveness
                .unwrap_or(0.0),
        }
    }

    fn calculate_error_detection_rate(&self) -> f32 {
        // Count errors caught by gates in validated run
        let mut errors_caught = 0;
        let mut total_gates = 0;

        for init in &self.validated_run.initiatives {
            for task in &init.tasks {
                if let Some(gate) = &task.validation_gate {
                    total_gates += 1;
                    match gate.gate_decision {
                        GateDecision::RequiresRework | GateDecision::Rejected => errors_caught += 1,
                        GateDecision::Approved => {}
                    }
                }
            }
        }

        if total_gates == 0 {
            return 0.0;
        }

        (errors_caught as f32 / total_gates as f32) * 100.0
    }
}

fn calculate_token_overhead(autonomous: &RunMetrics, validated: &RunMetrics) -> f32 {
    if autonomous.total_tokens == 0 {
        return 0.0;
    }

    let diff = validated.total_tokens as i64 - autonomous.total_tokens as i64;
    (diff as f32 / autonomous.total_tokens as f32) * 100.0
}

fn calculate_quality_delta(autonomous: &RunMetrics, validated: &RunMetrics) -> f32 {
    let autonomous_avg = (autonomous.avg_code_quality
        + autonomous.avg_test_coverage
        + autonomous.avg_doc_accuracy
        + autonomous.avg_instruction_adherence)
        / 4.0;
    let validated_avg = (validated.avg_code_quality
        + validated.avg_test_coverage
        + validated.avg_doc_accuracy
        + validated.avg_instruction_adherence)
        / 4.0;

    validated_avg - autonomous_avg
}

fn calculate_roi(token_overhead: f32, quality_delta: f32) -> f32 {
    if token_overhead == 0.0 {
        return 0.0;
    }

    quality_delta / (token_overhead / 100.0)
}

#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub token_overhead: f32,       // % increase in tokens for validated run
    pub quality_delta: f32,        // Quality score improvement (0-100)
    pub roi: f32,                  // Quality improvement per 1% token overhead
    pub error_detection_rate: f32, // % of tasks where gates caught issues
    pub gate_effectiveness: f32,   // % of gates that found issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_overhead_calculation() {
        let autonomous = RunMetrics {
            total_tokens: 1000,
            total_time: std::time::Duration::from_secs(100),
            avg_code_quality: 75.0,
            avg_test_coverage: 80.0,
            avg_doc_accuracy: 70.0,
            avg_instruction_adherence: 85.0,
            gate_effectiveness: None,
        };

        let validated = RunMetrics {
            total_tokens: 1100,
            total_time: std::time::Duration::from_secs(120),
            avg_code_quality: 80.0,
            avg_test_coverage: 85.0,
            avg_doc_accuracy: 75.0,
            avg_instruction_adherence: 90.0,
            gate_effectiveness: Some(50.0),
        };

        let overhead = calculate_token_overhead(&autonomous, &validated);
        assert_eq!(overhead, 10.0);
    }
}
