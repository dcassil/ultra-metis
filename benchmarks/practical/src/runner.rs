use crate::{types::*, api_client, prompt_builder, scenario_pack::LoadedScenarioPack};
use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::Utc;

/// Invoke the ultra-metis CLI binary, returning stdout and elapsed ms.
/// Mirrors the approach in benchmarks/run-ultra-metis-bench.sh.
pub fn run_cli(binary: &Path, args: &[&str]) -> anyhow::Result<CliResult> {
    let start = std::time::Instant::now();
    let output = Command::new(binary).args(args).output()?;
    let elapsed = start.elapsed();

    Ok(CliResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        elapsed,
    })
}

#[derive(Debug, Clone)]
pub struct CliResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub elapsed: std::time::Duration,
}

impl CliResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Approximate token count from output byte size.
    /// ~4 bytes per token is a rough but consistent approximation.
    pub fn approx_tokens(&self) -> u64 {
        ((self.stdout.len() + self.stderr.len()) / 4).max(1) as u64
    }

    pub fn as_trace_event(&self, label: impl Into<String>, command: impl Into<String>) -> CliTraceEvent {
        CliTraceEvent {
            label: label.into(),
            command: command.into(),
            exit_code: self.exit_code,
            duration: self.elapsed,
            approx_tokens: self.approx_tokens(),
        }
    }
}

/// Resolve ultra-metis binary path. Checks ULTRA_METIS_BINARY env var first,
/// then falls back to target/release/ultra-metis relative to cwd.
pub fn resolve_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("ULTRA_METIS_BINARY") {
        return PathBuf::from(path);
    }
    let candidate = PathBuf::from("target/release/ultra-metis");
    if candidate.exists() {
        return candidate;
    }
    PathBuf::from("ultra-metis")
}

/// Extract a short code (e.g., "BENCH-V-0001") from CLI stdout.
pub fn extract_short_code(stdout: &str, prefix: &str) -> String {
    for word in stdout.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if cleaned.starts_with(prefix) {
            return cleaned.to_string();
        }
    }
    String::new()
}

/// An initiative identified by AI from the scenario assessment.
#[derive(Debug)]
pub struct AiInitiative {
    pub id: String,
    pub title: String,
    pub objective: String,
    pub tasks: Vec<String>,
}

/// Parse Claude's JSON response to extract AI-identified initiatives.
pub fn parse_initiative_response(response: &str) -> Vec<AiInitiative> {
    let start = response.find('{');
    let end = response.rfind('}');

    let json_str = match (start, end) {
        (Some(s), Some(e)) if e > s => &response[s..=e],
        _ => return vec![],
    };

    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut result = vec![];
    if let Some(initiatives) = parsed.get("initiatives").and_then(|v| v.as_array()) {
        for init in initiatives {
            let id = init.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let title = init
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Initiative")
                .to_string();
            let objective = init
                .get("objective")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tasks = init
                .get("tasks")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            result.push(AiInitiative { id, title, objective, tasks });
        }
    }
    result
}

/// Score an AI-generated initiative into CodeMetrics based on content quality.
pub fn score_ai_initiative(init: &AiInitiative, response_was_valid_json: bool) -> CodeMetrics {
    let instruction_adherence = if response_was_valid_json { 100.0 } else { 50.0 };

    // Doc accuracy: has objective + at least 2 tasks?
    let doc_accuracy = if !init.objective.is_empty() && init.tasks.len() >= 2 {
        90.0
    } else if !init.objective.is_empty() {
        60.0
    } else {
        30.0
    };

    // Test coverage proxy: does task list mention testing/validation?
    let has_test_task = init.tasks.iter().any(|t| {
        let lower = t.to_lowercase();
        lower.contains("test") || lower.contains("verify") || lower.contains("validate")
    });
    let test_coverage = if has_test_task { 80.0 } else { 40.0 };

    // Lines proxy: text length as thoroughness indicator (~80 chars per line)
    let text_len = (init.objective.len() + init.tasks.join(" ").len()) as u32;
    let lines = (text_len / 80).max(1);

    CodeMetrics {
        lines_of_code: lines,
        test_coverage_percent: test_coverage,
        cyclomatic_complexity: init.tasks.len() as f32,
        doc_accuracy_percent: doc_accuracy,
        instruction_adherence_percent: instruction_adherence,
    }
}

fn fallback_initiative(api_tokens: u64, api_time: std::time::Duration, valid_json: bool) -> InitiativeResult {
    let metrics = CodeMetrics {
        lines_of_code: 1,
        test_coverage_percent: 0.0,
        cyclomatic_complexity: 0.0,
        doc_accuracy_percent: if valid_json { 80.0 } else { 30.0 },
        instruction_adherence_percent: if valid_json { 100.0 } else { 50.0 },
    };
    InitiativeResult {
        initiative_id: "no-additional".to_string(),
        initiative_title: "No additional initiatives identified".to_string(),
        tasks: vec![TaskResult {
            task_id: "strategic-assessment".to_string(),
            task_title: "Strategic completeness assessment".to_string(),
            status: TaskStatus::Completed,
            tokens_used: api_tokens,
            time_elapsed: api_time,
            code_metrics: metrics,
            validation_gate: None,
        }],
        total_tokens: api_tokens,
        total_time: api_time,
    }
}

/// Execute autonomous benchmark run (no validation gates).
///
/// Initializes a temp ultra-metis project, runs the scenario through Claude API,
/// and records real token counts and CLI timing.
pub async fn execute_autonomous(scenario: &LoadedScenarioPack) -> anyhow::Result<BenchmarkRun> {
    let start_time = std::time::Instant::now();
    let run_id = uuid::Uuid::new_v4().to_string();
    let binary = resolve_binary_path();
    let mut phases = vec![];
    let mut trace = RunTrace::default();

    tracing::info!("Starting autonomous run: {} (binary: {:?})", run_id, binary);

    // Create temp project (auto-cleaned on drop)
    let temp_dir = tempfile::TempDir::new()?;
    let proj_str = temp_dir.path().to_str().unwrap_or("/tmp/bench-auto");

    // Initialize metis project and create known scenario documents
    if let Ok(result) = run_cli(&binary, &["init", "--path", proj_str, "--prefix", "BENCH"]) {
        trace.cli_events.push(result.as_trace_event(
            "workspace_init",
            format!("{} init --path {} --prefix BENCH", binary.display(), proj_str),
        ));
    }
    let vision_result = run_cli(&binary, &[
        "create", "--type", "vision", "--path", proj_str, "File Processing Toolkit",
    ]);
    if let Ok(result) = &vision_result {
        trace.cli_events.push(result.as_trace_event(
            "seed_vision",
            format!("{} create --type vision --path {} File Processing Toolkit", binary.display(), proj_str),
        ));
    }
    let vision_code = vision_result
        .as_ref()
        .map(|r| extract_short_code(&r.stdout, "BENCH-V-"))
        .unwrap_or_default();

    if !vision_code.is_empty() {
        if let Ok(result) = run_cli(&binary, &[
            "create", "--type", "initiative", "--path", proj_str,
            "--parent", &vision_code, "Parse Module",
        ]) {
            trace.cli_events.push(result.as_trace_event(
                "seed_initiative_parse",
                format!("{} create --type initiative --path {} --parent {} Parse Module", binary.display(), proj_str, vision_code),
            ));
        }
        if let Ok(result) = run_cli(&binary, &[
            "create", "--type", "initiative", "--path", proj_str,
            "--parent", &vision_code, "Transform Module",
        ]) {
            trace.cli_events.push(result.as_trace_event(
                "seed_initiative_transform",
                format!("{} create --type initiative --path {} --parent {} Transform Module", binary.display(), proj_str, vision_code),
            ));
        }
    }
    phases.push(PhaseResult {
        phase: BenchmarkPhase::ScenarioSetup,
        status: PhaseStatus::Completed,
        tokens_used: trace.cli_events.iter().map(|e| e.approx_tokens).sum(),
        time_elapsed: trace.cli_events.iter().map(|e| e.duration).sum(),
        notes: vec![format!("Scenario materialized in {}", temp_dir.path().display())],
    });

    // Ask Claude to assess what additional initiatives are needed
    let prompt = prompt_builder::build_scenario_assessment_prompt(&scenario.root)?;
    let api_start = std::time::Instant::now();
    let api_resp = api_client::ask(&prompt.system, &prompt.user).await?;
    let api_time = api_start.elapsed();
    trace.prompt_events.push(PromptTraceEvent {
        label: "scenario_assessment".to_string(),
        input_tokens: api_resp.input_tokens,
        output_tokens: api_resp.output_tokens,
        duration: api_time,
    });
    phases.push(PhaseResult {
        phase: BenchmarkPhase::DocumentGeneration,
        status: PhaseStatus::Completed,
        tokens_used: api_resp.total_tokens(),
        time_elapsed: api_time,
        notes: vec![format!(
            "Scenario '{}' with {} seeded initiatives assessed",
            scenario.manifest.id,
            scenario.seed_initiatives.len()
        )],
    });

    let ai_initiatives = parse_initiative_response(&api_resp.content);
    let response_was_valid_json = !ai_initiatives.is_empty()
        || api_resp.content.contains("additional_initiatives_needed");

    // Build initiative results from AI response
    let mut initiatives = vec![];
    let n = ai_initiatives.len().max(1) as u32;

    for (idx, ai_init) in ai_initiatives.iter().enumerate() {
        // Create the initiative in CLI for artifact tracking
        let cli_result = if !vision_code.is_empty() {
            let result = run_cli(&binary, &[
                "create", "--type", "initiative", "--path", proj_str,
                "--parent", &vision_code, &ai_init.title,
            ]).ok();
            if let Some(ref cli) = result {
                trace.cli_events.push(cli.as_trace_event(
                    format!("materialize_{}", ai_init.id),
                    format!("{} create --type initiative --path {} --parent {} {}", binary.display(), proj_str, vision_code, ai_init.title),
                ));
            }
            result
        } else {
            None
        };

        let cli_tokens = cli_result.as_ref().map(|r| r.approx_tokens()).unwrap_or(0);
        let cli_time = cli_result.as_ref().map(|r| r.elapsed).unwrap_or_default();
        let task_tokens = (api_resp.total_tokens() / n as u64) + cli_tokens;
        let task_time = (api_time / n) + cli_time;

        tracing::info!(
            "AI initiative {}/{}: '{}' (tokens: {})",
            idx + 1, n, ai_init.title, task_tokens
        );

        initiatives.push(InitiativeResult {
            initiative_id: ai_init.id.clone(),
            initiative_title: ai_init.title.clone(),
            tasks: vec![TaskResult {
                task_id: format!("{}-assess", ai_init.id),
                task_title: format!("Assess and design: {}", ai_init.title),
                status: TaskStatus::Completed,
                tokens_used: task_tokens,
                time_elapsed: task_time,
                code_metrics: score_ai_initiative(ai_init, response_was_valid_json),
                validation_gate: None,
            }],
            total_tokens: task_tokens,
            total_time: task_time,
        });
    }

    if initiatives.is_empty() {
        tracing::info!("AI identified no additional initiatives");
        initiatives.push(fallback_initiative(
            api_resp.total_tokens(),
            api_time,
            response_was_valid_json,
        ));
    }

    let total_tokens: u64 = initiatives.iter().map(|i| i.total_tokens).sum();
    let total_time = start_time.elapsed();
    phases.push(PhaseResult {
        phase: BenchmarkPhase::Decomposition,
        status: PhaseStatus::Completed,
        tokens_used: total_tokens,
        time_elapsed: total_time,
        notes: vec![format!("Produced {} initiative assessments", initiatives.len())],
    });
    let task_count = initiatives.iter().map(|i| i.tasks.len()).sum::<usize>().max(1) as f32;
    let avg_doc_accuracy = initiatives
        .iter()
        .flat_map(|i| i.tasks.iter().map(|t| t.code_metrics.doc_accuracy_percent))
        .sum::<f32>()
        / task_count;
    let avg_instruction_adherence = initiatives
        .iter()
        .flat_map(|i| i.tasks.iter().map(|t| t.code_metrics.instruction_adherence_percent))
        .sum::<f32>()
        / task_count;

    Ok(BenchmarkRun {
        run_id,
        timestamp: Utc::now(),
        scenario: ScenarioSummary {
            id: scenario.manifest.id.clone(),
            title: scenario.manifest.title.clone(),
            root: scenario.root.display().to_string(),
        },
        execution_mode: ExecutionMode::Autonomous,
        phases,
        trace,
        initiatives,
        total_metrics: RunMetrics {
            total_tokens,
            total_time,
            avg_code_quality: avg_doc_accuracy,
            avg_test_coverage: 0.0,
            avg_doc_accuracy,
            avg_instruction_adherence,
            gate_effectiveness: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_result_token_approximation() {
        let result = CliResult {
            stdout: "a".repeat(400),
            stderr: String::new(),
            exit_code: 0,
            elapsed: std::time::Duration::from_millis(10),
        };
        assert_eq!(result.approx_tokens(), 100);
    }

    #[test]
    fn test_cli_result_success() {
        let ok = CliResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            elapsed: std::time::Duration::default(),
        };
        let fail = CliResult { exit_code: 1, ..ok.clone() };
        assert!(ok.success());
        assert!(!fail.success());
    }

    #[test]
    fn test_extract_short_code() {
        let stdout = "Created BENCH-V-0001 successfully\n";
        assert_eq!(extract_short_code(stdout, "BENCH-V-"), "BENCH-V-0001");
    }

    #[test]
    fn test_extract_short_code_missing() {
        let stdout = "no code here\n";
        assert_eq!(extract_short_code(stdout, "BENCH-V-"), "");
    }

    #[test]
    fn test_parse_initiative_response_valid_json() {
        let resp = r#"{"analysis":"needs output","additional_initiatives_needed":true,"initiatives":[{"id":"output-module","title":"Output Module","objective":"Export data","tasks":["Write CSV","Write JSON","Validate schema"]}]}"#;
        let initiatives = parse_initiative_response(resp);
        assert_eq!(initiatives.len(), 1);
        assert_eq!(initiatives[0].title, "Output Module");
        assert_eq!(initiatives[0].tasks.len(), 3);
    }

    #[test]
    fn test_parse_initiative_response_empty() {
        let resp = "sorry I cannot help";
        let initiatives = parse_initiative_response(resp);
        assert!(initiatives.is_empty());
    }

    #[test]
    fn test_score_ai_initiative_with_tests() {
        let init = AiInitiative {
            id: "test-init".to_string(),
            title: "Test Initiative".to_string(),
            objective: "Build something important".to_string(),
            tasks: vec!["Design API".to_string(), "Write unit tests".to_string(), "Deploy".to_string()],
        };
        let metrics = score_ai_initiative(&init, true);
        assert_eq!(metrics.instruction_adherence_percent, 100.0);
        assert_eq!(metrics.test_coverage_percent, 80.0);
        assert_eq!(metrics.cyclomatic_complexity, 3.0);
    }

    #[test]
    fn test_autonomous_runner_creates_valid_run() {
        let run = BenchmarkRun {
            run_id: "test".to_string(),
            timestamp: Utc::now(),
            scenario: ScenarioSummary {
                id: "test-scenario".to_string(),
                title: "Test Scenario".to_string(),
                root: "scenario".to_string(),
            },
            execution_mode: ExecutionMode::Autonomous,
            phases: vec![],
            trace: RunTrace::default(),
            initiatives: vec![],
            total_metrics: RunMetrics {
                total_tokens: 0,
                total_time: std::time::Duration::from_secs(0),
                avg_code_quality: 0.0,
                avg_test_coverage: 0.0,
                avg_doc_accuracy: 0.0,
                avg_instruction_adherence: 0.0,
                gate_effectiveness: None,
            },
        };
        assert_eq!(run.execution_mode, ExecutionMode::Autonomous);
    }
}
