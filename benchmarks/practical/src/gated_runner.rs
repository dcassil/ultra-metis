use crate::{
    api_client, gate_scorer::GateScorer, prompt_builder, runner, scenario_pack::LoadedScenarioPack,
    types::*, workspace,
};
use chrono::Utc;

/// Execute validated benchmark run with quality gates after each initiative.
///
/// Same flow as autonomous but each AI-identified initiative is reviewed by a
/// gate check prompt before being accepted. Rework tokens are tracked.
pub async fn execute_with_gates(scenario: &LoadedScenarioPack) -> anyhow::Result<BenchmarkRun> {
    let start_time = std::time::Instant::now();
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut phases = vec![];
    let mut trace = RunTrace::default();
    let manifest = workspace::default_manifest(scenario, SystemUnderTest::UltraMetis);

    tracing::info!("Starting validated run with gates: {}", run_id);

    // Create isolated workspace and seed scenario documents
    let mut ws = workspace::BenchmarkWorkspace::setup(scenario)?;
    let setup = ws.take_setup_trace();
    trace.cli_events.extend(setup.cli_events);
    phases.push(setup.phase_result);

    // Ask Claude to assess what additional initiatives are needed
    let prompt = prompt_builder::build_scenario_assessment_prompt(&scenario.root)?;
    let api_start = std::time::Instant::now();
    let api_result = api_client::ask(&prompt.system, &prompt.user).await;
    let api_time = api_start.elapsed();
    let (
        api_input_tokens,
        api_output_tokens,
        api_content,
        ai_initiatives,
        response_was_valid_json,
        phase_note,
    ) = match api_result {
        Ok(api_resp) => {
            let parsed = runner::parse_initiative_response(&api_resp.content);
            let valid_json =
                !parsed.is_empty() || api_resp.content.contains("additional_initiatives_needed");
            (
                api_resp.input_tokens,
                api_resp.output_tokens,
                api_resp.content,
                parsed,
                valid_json,
                format!(
                    "Scenario '{}' with {} seeded initiatives assessed",
                    scenario.manifest.id,
                    scenario.seed_initiatives.len()
                ),
            )
        }
        Err(err) => (
            0,
            0,
            format!("Deterministic fallback used because scenario assessment failed: {err}"),
            vec![runner::default_fallback_ai_initiative()],
            false,
            format!("Fallback initiative generation used after Claude failure: {err}"),
        ),
    };
    trace.prompt_events.push(runner::prompt_trace_event(
        "scenario_assessment",
        &prompt.system,
        &prompt.user,
        &api_content,
        api_input_tokens,
        api_output_tokens,
        api_time,
    ));
    phases.push(PhaseResult {
        phase: BenchmarkPhase::DocumentGeneration,
        status: PhaseStatus::Completed,
        tokens_used: api_input_tokens + api_output_tokens,
        time_elapsed: api_time,
        notes: vec![phase_note],
    });

    let scorer = GateScorer::new();
    let mut initiatives = vec![];
    let mut total_rework_tokens = 0u64;
    let n = ai_initiatives.len().max(1) as u32;

    for (idx, ai_init) in ai_initiatives.iter().enumerate() {
        // Create the initiative in CLI
        let cli_result = ws.create_initiative(&ai_init.title);
        if let Some(ref cli) = cli_result {
            trace.cli_events.push(cli.as_trace_event(
                format!("materialize_{}", ai_init.id),
                format!("create initiative: {}", ai_init.title),
            ));
        }

        let cli_tokens = cli_result.as_ref().map(|r| r.approx_tokens()).unwrap_or(0);
        let cli_time = cli_result.as_ref().map(|r| r.elapsed).unwrap_or_default();
        let task_tokens = ((api_input_tokens + api_output_tokens) / n as u64) + cli_tokens;
        let task_time = (api_time / n) + cli_time;

        // Build a partial InitiativeResult so GateScorer can inspect its structure
        let code_metrics = runner::score_ai_initiative(ai_init, response_was_valid_json);
        let partial = InitiativeResult {
            initiative_id: ai_init.id.clone(),
            initiative_title: ai_init.title.clone(),
            tasks: vec![TaskResult {
                task_id: format!("{}-assess", ai_init.id),
                task_title: format!("Assess and design: {}", ai_init.title),
                status: TaskStatus::Completed,
                tokens_used: task_tokens,
                time_elapsed: task_time,
                code_metrics: code_metrics.clone(),
                validation_gate: None,
            }],
            total_tokens: task_tokens,
            total_time: task_time,
        };

        // 1. Structural gate (deterministic, no API cost)
        let structural = scorer.score_initiative(&partial, None);
        let structural_rejected = matches!(structural.gate_decision, GateDecision::Rejected);

        // 2. Semantic gate via Claude API (only when structure looks sound)
        let validation_gate = if structural_rejected {
            tracing::info!(
                "Structural gate rejected initiative {}/{}: '{}' (issues: {})",
                idx + 1,
                n,
                ai_init.title,
                structural.issues_found.len()
            );
            total_rework_tokens += structural.rework_tokens;
            Some(structural)
        } else {
            let initiative_content = format!(
                "Objective: {}\nTasks: {}",
                ai_init.objective,
                ai_init.tasks.join(", ")
            );
            let api_gate = run_gate_check(&ai_init.title, &initiative_content).await;
            match api_gate {
                Ok(outcome) => {
                    trace.prompt_events.push(outcome.trace_event);
                    // Merge structural issues into API gate result
                    let mut api_issues = outcome.issues;
                    api_issues.extend(structural.issues_found.clone());
                    let merged_decision =
                        stricter_decision(outcome.decision, structural.gate_decision);
                    let merged_rework = outcome.rework_tokens + structural.rework_tokens;
                    total_rework_tokens += merged_rework;
                    tracing::info!(
                        "Gate check {}/{}: '{}' → {:?} (rework tokens: {})",
                        idx + 1,
                        n,
                        ai_init.title,
                        merged_decision,
                        merged_rework
                    );
                    Some(ValidationGateResult {
                        gate_decision: merged_decision,
                        issues_found: api_issues,
                        rework_tokens: merged_rework,
                        rework_time: outcome.rework_time,
                    })
                }
                Err(e) => {
                    // API unavailable — fall back to structural gate
                    tracing::warn!(
                        "API gate failed for '{}', using structural gate: {}",
                        ai_init.title,
                        e
                    );
                    total_rework_tokens += structural.rework_tokens;
                    Some(structural)
                }
            }
        };

        initiatives.push(InitiativeResult {
            initiative_id: ai_init.id.clone(),
            initiative_title: ai_init.title.clone(),
            tasks: vec![TaskResult {
                task_id: format!("{}-assess", ai_init.id),
                task_title: format!("Assess and design: {}", ai_init.title),
                status: TaskStatus::Completed,
                tokens_used: task_tokens,
                time_elapsed: task_time,
                code_metrics,
                validation_gate,
            }],
            total_tokens: task_tokens,
            total_time: task_time,
        });
    }

    // Handle case where AI found no additional initiatives
    if initiatives.is_empty() {
        tracing::info!("AI identified no additional initiatives — running gate on base response");
        let gate_result = run_gate_check("Strategic assessment", &api_content).await;
        let validation_gate = match gate_result {
            Ok(outcome) => {
                trace.prompt_events.push(outcome.trace_event);
                total_rework_tokens += outcome.rework_tokens;
                Some(ValidationGateResult {
                    gate_decision: outcome.decision,
                    issues_found: outcome.issues,
                    rework_tokens: outcome.rework_tokens,
                    rework_time: outcome.rework_time,
                })
            }
            Err(_) => None,
        };

        let metrics = CodeMetrics {
            lines_of_code: 1,
            test_coverage_percent: 0.0,
            cyclomatic_complexity: 0.0,
            doc_accuracy_percent: if response_was_valid_json { 80.0 } else { 30.0 },
            instruction_adherence_percent: if response_was_valid_json { 100.0 } else { 50.0 },
        };
        initiatives.push(InitiativeResult {
            initiative_id: "no-additional".to_string(),
            initiative_title: "No additional initiatives identified".to_string(),
            tasks: vec![TaskResult {
                task_id: "strategic-assessment".to_string(),
                task_title: "Strategic completeness assessment".to_string(),
                status: TaskStatus::Completed,
                tokens_used: api_input_tokens + api_output_tokens,
                time_elapsed: api_time,
                code_metrics: metrics,
                validation_gate,
            }],
            total_tokens: api_input_tokens + api_output_tokens,
            total_time: api_time,
        });
    }

    let total_tokens: u64 =
        initiatives.iter().map(|i| i.total_tokens).sum::<u64>() + total_rework_tokens;
    let total_time = start_time.elapsed();
    phases.push(PhaseResult {
        phase: BenchmarkPhase::Decomposition,
        status: PhaseStatus::Completed,
        tokens_used: total_tokens,
        time_elapsed: total_time,
        notes: vec![format!(
            "Produced {} gated initiative assessments",
            initiatives.len()
        )],
    });
    let gate_effectiveness = calculate_gate_effectiveness(&initiatives);

    let task_count = initiatives
        .iter()
        .map(|i| i.tasks.len())
        .sum::<usize>()
        .max(1) as f32;
    let avg_doc_accuracy = initiatives
        .iter()
        .flat_map(|i| i.tasks.iter().map(|t| t.code_metrics.doc_accuracy_percent))
        .sum::<f32>()
        / task_count;
    let avg_instruction_adherence = initiatives
        .iter()
        .flat_map(|i| {
            i.tasks
                .iter()
                .map(|t| t.code_metrics.instruction_adherence_percent)
        })
        .sum::<f32>()
        / task_count;

    Ok(BenchmarkRun {
        run_id,
        timestamp: Utc::now(),
        manifest,
        scenario: ScenarioSummary {
            id: scenario.manifest.id.clone(),
            title: scenario.manifest.title.clone(),
            root: scenario.root.display().to_string(),
        },
        execution_mode: ExecutionMode::Validated,
        phases,
        trace,
        artifacts: runner::snapshot_workspace_artifacts(ws.path()),
        initiatives,
        total_metrics: RunMetrics {
            total_tokens,
            total_time,
            avg_code_quality: avg_doc_accuracy,
            avg_test_coverage: 0.0,
            avg_doc_accuracy,
            avg_instruction_adherence,
            gate_effectiveness: Some(gate_effectiveness),
        },
    })
}

/// Return the stricter of two gate decisions (Rejected > RequiresRework > Approved).
fn stricter_decision(a: GateDecision, b: GateDecision) -> GateDecision {
    match (a, b) {
        (GateDecision::Rejected, _) | (_, GateDecision::Rejected) => GateDecision::Rejected,
        (GateDecision::RequiresRework, _) | (_, GateDecision::RequiresRework) => {
            GateDecision::RequiresRework
        }
        _ => GateDecision::Approved,
    }
}

/// Run a quality gate check via Claude API on an initiative description.
/// Returns (decision, issues, rework_tokens, rework_time).
async fn run_gate_check(
    initiative_title: &str,
    initiative_content: &str,
) -> anyhow::Result<GateCheckOutcome> {
    let gate_prompt = prompt_builder::build_gate_check_prompt(initiative_title, initiative_content);
    let gate_start = std::time::Instant::now();
    let gate_resp = api_client::ask(&gate_prompt.system, &gate_prompt.user).await?;
    let gate_time = gate_start.elapsed();

    let (decision, issues) = parse_gate_response(&gate_resp.content);

    Ok(GateCheckOutcome {
        decision,
        issues,
        rework_tokens: gate_resp.total_tokens(),
        rework_time: gate_time,
        trace_event: runner::prompt_trace_event(
            format!(
                "gate_check_{}",
                initiative_title.to_lowercase().replace(' ', "_")
            ),
            gate_prompt.system,
            gate_prompt.user,
            &gate_resp.content,
            gate_resp.input_tokens,
            gate_resp.output_tokens,
            gate_time,
        ),
    })
}

struct GateCheckOutcome {
    decision: GateDecision,
    issues: Vec<String>,
    rework_tokens: u64,
    rework_time: std::time::Duration,
    trace_event: PromptTraceEvent,
}

fn parse_gate_response(response: &str) -> (GateDecision, Vec<String>) {
    let start = response.find('{');
    let end = response.rfind('}');

    let json_str = match (start, end) {
        (Some(s), Some(e)) if e > s => &response[s..=e],
        _ => return (GateDecision::Approved, vec![]),
    };

    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return (GateDecision::Approved, vec![]),
    };

    let approved = parsed
        .get("approved")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let score = parsed.get("score").and_then(|v| v.as_f64()).unwrap_or(0.8);
    let issues: Vec<String> = parsed
        .get("issues")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|i| i.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let decision = if approved && score >= 0.7 {
        GateDecision::Approved
    } else if score >= 0.4 {
        GateDecision::RequiresRework
    } else {
        GateDecision::Rejected
    };

    (decision, issues)
}

/// Percentage of gates that caught at least one issue (0-100%).
fn calculate_gate_effectiveness(initiatives: &[InitiativeResult]) -> f32 {
    let mut gates_with_issues = 0usize;
    let mut total_gates = 0usize;

    for init in initiatives {
        for task in &init.tasks {
            if let Some(gate) = &task.validation_gate {
                total_gates += 1;
                if !gate.issues_found.is_empty() {
                    gates_with_issues += 1;
                }
            }
        }
    }

    if total_gates == 0 {
        return 0.0;
    }

    (gates_with_issues as f32 / total_gates as f32) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_effectiveness_calculation() {
        let effectiveness = calculate_gate_effectiveness(&[]);
        assert_eq!(effectiveness, 0.0);
    }

    #[test]
    fn test_parse_gate_response_approved() {
        let resp = r#"{"approved":true,"score":0.9,"issues":[]}"#;
        let (decision, issues) = parse_gate_response(resp);
        assert!(matches!(decision, GateDecision::Approved));
        assert!(issues.is_empty());
    }

    #[test]
    fn test_parse_gate_response_requires_rework() {
        let resp = r#"{"approved":false,"score":0.5,"issues":["Missing acceptance criteria","No risk section"]}"#;
        let (decision, issues) = parse_gate_response(resp);
        assert!(matches!(decision, GateDecision::RequiresRework));
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_parse_gate_response_invalid_json() {
        let (decision, issues) = parse_gate_response("not json at all");
        assert!(matches!(decision, GateDecision::Approved));
        assert!(issues.is_empty());
    }
}
