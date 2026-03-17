use crate::types::*;
use std::path::Path;
use chrono::Utc;

/// Execute validated benchmark run with quality gates after each initiative
pub async fn execute_with_gates(scenario_path: &Path) -> anyhow::Result<BenchmarkRun> {
    let start_time = std::time::Instant::now();
    let run_id = uuid::Uuid::new_v4().to_string();

    tracing::info!("Starting validated run with gates: {}", run_id);

    let mut initiatives = vec![];
    let mut total_rework_tokens = 0u64;

    // Execute first initiative through design, implementation, testing
    let parse_init = execute_initiative_with_gate(
        "parse-module",
        "Parse Module",
        scenario_path,
    ).await?;

    if let Some(gate) = &parse_init.tasks.first().and_then(|t| t.validation_gate.as_ref()) {
        total_rework_tokens += gate.rework_tokens;
    }
    initiatives.push(parse_init);

    // Execute second initiative with gate
    let transform_init = execute_initiative_with_gate(
        "transform-module",
        "Transform Module",
        scenario_path,
    ).await?;

    if let Some(gate) = &transform_init.tasks.first().and_then(|t| t.validation_gate.as_ref()) {
        total_rework_tokens += gate.rework_tokens;
    }
    initiatives.push(transform_init);

    let total_tokens: u64 = initiatives.iter().map(|i| i.total_tokens).sum::<u64>() + total_rework_tokens;
    let total_time = start_time.elapsed();

    // Calculate gate effectiveness (what % of issues were caught?)
    let gate_effectiveness = calculate_gate_effectiveness(&initiatives);

    Ok(BenchmarkRun {
        run_id,
        timestamp: Utc::now(),
        execution_mode: ExecutionMode::Validated,
        initiatives,
        total_metrics: RunMetrics {
            total_tokens,
            total_time,
            avg_code_quality: 0.0,
            avg_test_coverage: 0.0,
            avg_doc_accuracy: 0.0,
            avg_instruction_adherence: 0.0,
            gate_effectiveness: Some(gate_effectiveness),
        },
    })
}

async fn execute_initiative_with_gate(
    initiative_id: &str,
    initiative_title: &str,
    _scenario_path: &Path,
) -> anyhow::Result<InitiativeResult> {
    // TODO: Execute initiative, then run validation gate
    // For now, return placeholder

    Ok(InitiativeResult {
        initiative_id: initiative_id.to_string(),
        initiative_title: initiative_title.to_string(),
        tasks: vec![
            TaskResult {
                task_id: format!("{}-task-1", initiative_id),
                task_title: "Design and specification".to_string(),
                status: TaskStatus::Completed,
                tokens_used: 2000,
                time_elapsed: std::time::Duration::from_secs(60),
                code_metrics: CodeMetrics {
                    lines_of_code: 500,
                    test_coverage_percent: 85.0,
                    cyclomatic_complexity: 2.5,
                    doc_accuracy_percent: 90.0,
                    instruction_adherence_percent: 95.0,
                },
                validation_gate: Some(ValidationGateResult {
                    gate_decision: GateDecision::Approved,
                    issues_found: vec![],
                    rework_tokens: 0,
                    rework_time: std::time::Duration::from_secs(0),
                }),
            },
        ],
        total_tokens: 2000,
        total_time: std::time::Duration::from_secs(60),
    })
}

fn calculate_gate_effectiveness(initiatives: &[InitiativeResult]) -> f32 {
    let mut issues_found = 0;
    let mut total_gates = 0;

    for init in initiatives {
        for task in &init.tasks {
            if let Some(gate) = &task.validation_gate {
                total_gates += 1;
                if !gate.issues_found.is_empty() {
                    issues_found += gate.issues_found.len();
                }
            }
        }
    }

    if total_gates == 0 {
        return 0.0;
    }

    (issues_found as f32 / total_gates as f32) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_effectiveness_calculation() {
        let effectiveness = calculate_gate_effectiveness(&[]);
        assert_eq!(effectiveness, 0.0);
    }
}
