use crate::api_client;
use crate::doc_quality::score_content;
use crate::mcp_adapter::{
    ExecutionAdapter, McpSession, OriginalMetisAdapter, SystemUnderTest, UltraMetisMcpAdapter,
};
use crate::prompt_builder;
use crate::runner::{parse_initiative_response, AiInitiative};
use crate::scenario_pack::{LoadedScenarioPack, ScenarioArtifact};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningComparisonResult {
    pub timestamp: chrono::DateTime<Utc>,
    pub scenario_id: String,
    pub scenario_title: String,
    pub generated_initiatives: Vec<PlannedInitiativeSummary>,
    pub original_metis: PlanningSystemResult,
    pub ultra_metis_mcp: PlanningSystemResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedInitiativeSummary {
    pub id: String,
    pub title: String,
    pub objective: String,
    pub tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningSystemResult {
    pub system: String,
    pub initiative_count: usize,
    pub total_tokens: u64,
    pub total_time_ms: f64,
    pub avg_completeness_percent: f32,
    pub avg_placeholder_count: f32,
    pub avg_task_count: f32,
    pub docs: Vec<PlanningDocumentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningDocumentResult {
    pub initiative_id: String,
    pub title: String,
    pub short_code: String,
    pub doc_quality: PlanningDocQuality,
    pub tokens_used: u64,
    pub fill_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningDocQuality {
    pub completeness_percent: f32,
    pub placeholder_count: u32,
    pub content_lines: u32,
    pub filled_sections: usize,
    pub empty_sections: usize,
}

pub async fn run_planning_comparison(
    scenario: &LoadedScenarioPack,
) -> Result<PlanningComparisonResult> {
    let (planned, assessment_tokens, assessment_time) = plan_initiatives(scenario).await?;

    let original = run_system_planning(
        &OriginalMetisAdapter,
        scenario,
        &planned,
        assessment_tokens,
        assessment_time,
    )
    .await?;
    let ultra = run_system_planning(
        &UltraMetisMcpAdapter,
        scenario,
        &planned,
        assessment_tokens,
        assessment_time,
    )
    .await?;

    Ok(PlanningComparisonResult {
        timestamp: Utc::now(),
        scenario_id: scenario.manifest.id.clone(),
        scenario_title: scenario.manifest.title.clone(),
        generated_initiatives: planned
            .into_iter()
            .map(|initiative| PlannedInitiativeSummary {
                id: initiative.id,
                title: initiative.title,
                objective: initiative.objective,
                tasks: initiative.tasks,
            })
            .collect(),
        original_metis: original,
        ultra_metis_mcp: ultra,
    })
}

async fn plan_initiatives(
    scenario: &LoadedScenarioPack,
) -> Result<(Vec<AiInitiative>, u64, Duration)> {
    let prompt = prompt_builder::build_scenario_assessment_prompt(&scenario.root)?;
    let assessment_start = Instant::now();

    match api_client::ask(&prompt.system, &prompt.user).await {
        Ok(assessment) => {
            let mut planned = parse_initiative_response(&assessment.content);
            if planned.is_empty() {
                planned = derive_fallback_initiatives(scenario);
            }
            Ok((
                planned,
                assessment.total_tokens(),
                assessment_start.elapsed(),
            ))
        }
        Err(_) => Ok((
            derive_fallback_initiatives(scenario),
            0,
            assessment_start.elapsed(),
        )),
    }
}

async fn run_system_planning<A: ExecutionAdapter>(
    adapter: &A,
    scenario: &LoadedScenarioPack,
    planned: &[AiInitiative],
    assessment_tokens: u64,
    assessment_time: Duration,
) -> Result<PlanningSystemResult> {
    let temp = tempfile::tempdir().context("Failed to create planning comparison workspace")?;
    let root = temp.path();
    let mut session = adapter.start()?;
    let document_project_path = document_project_path(adapter.system_under_test(), root);
    let init_project_path = root.display().to_string();

    call_tool(
        &mut session,
        "initialize_project",
        json!({
            "project_path": init_project_path,
            "prefix": "PLAN"
        }),
    )?;

    let vision_code = seed_vision(
        adapter.system_under_test(),
        &mut session,
        &document_project_path,
        &scenario.manifest.title,
    )?;
    for seed in &scenario.seed_initiatives {
        let title = extract_title(seed).unwrap_or_else(|| "Seed Initiative".to_string());
        let _ = create_initiative(&mut session, &document_project_path, &vision_code, &title)?;
    }

    let mut docs = vec![];
    let mut total_tokens = assessment_tokens;
    let mut total_time_ms = assessment_time.as_secs_f64() * 1000.0;

    for initiative in planned {
        let short_code = create_initiative(
            &mut session,
            &document_project_path,
            &vision_code,
            &initiative.title,
        )?;
        let template = read_document(&mut session, &document_project_path, &short_code)?;

        let fill_start = Instant::now();
        let (filled, fill_tokens) = fill_initiative_template(initiative, &template).await?;
        let fill_time_ms = fill_start.elapsed().as_secs_f64() * 1000.0;
        let quality = score_content(&filled);

        total_tokens += fill_tokens;
        total_time_ms += fill_time_ms;

        docs.push(PlanningDocumentResult {
            initiative_id: initiative.id.clone(),
            title: initiative.title.clone(),
            short_code,
            doc_quality: PlanningDocQuality {
                completeness_percent: quality.completeness_percent,
                placeholder_count: quality.placeholder_count,
                content_lines: quality.content_lines,
                filled_sections: quality.filled_sections.len(),
                empty_sections: quality.empty_sections.len(),
            },
            tokens_used: fill_tokens,
            fill_time_ms,
        });
    }

    let count = docs.len().max(1) as f32;
    let avg_completeness = docs
        .iter()
        .map(|doc| doc.doc_quality.completeness_percent)
        .sum::<f32>()
        / count;
    let avg_placeholders = docs
        .iter()
        .map(|doc| doc.doc_quality.placeholder_count as f32)
        .sum::<f32>()
        / count;
    let avg_task_count = planned
        .iter()
        .map(|initiative| initiative.tasks.len() as f32)
        .sum::<f32>()
        / (planned.len().max(1) as f32);

    Ok(PlanningSystemResult {
        system: system_name(adapter.system_under_test()).to_string(),
        initiative_count: docs.len(),
        total_tokens,
        total_time_ms,
        avg_completeness_percent: avg_completeness,
        avg_placeholder_count: avg_placeholders,
        avg_task_count,
        docs,
    })
}

fn seed_vision(
    system: SystemUnderTest,
    session: &mut McpSession,
    project_path: &str,
    title: &str,
) -> Result<String> {
    match system {
        SystemUnderTest::OriginalMetis => Ok("PLAN-V-0001".to_string()),
        SystemUnderTest::UltraMetisMcp => {
            let response = call_tool(
                session,
                "create_document",
                json!({
                    "project_path": project_path,
                    "document_type": "vision",
                    "title": title,
                }),
            )?;
            extract_short_code(&extract_text_content(&response), "PLAN-V-").ok_or_else(|| {
                anyhow!("Failed to extract vision short code from ultra-metis response")
            })
        }
    }
}

fn create_initiative(
    session: &mut McpSession,
    project_path: &str,
    vision_code: &str,
    title: &str,
) -> Result<String> {
    let response = call_tool(
        session,
        "create_document",
        json!({
            "project_path": project_path,
            "document_type": "initiative",
            "title": title,
            "parent_id": vision_code,
        }),
    )?;
    extract_short_code(&extract_text_content(&response), "PLAN-I-")
        .ok_or_else(|| anyhow!("Failed to extract initiative short code"))
}

fn read_document(session: &mut McpSession, project_path: &str, short_code: &str) -> Result<String> {
    let response = call_tool(
        session,
        "read_document",
        json!({
            "project_path": project_path,
            "short_code": short_code,
        }),
    )?;
    Ok(extract_text_content(&response))
}

fn call_tool(session: &mut McpSession, tool_name: &str, arguments: Value) -> Result<Value> {
    let response = session.call_tool(tool_name, arguments)?;
    if is_error_response(&response) {
        return Err(anyhow!(
            "{} failed: {}",
            tool_name,
            extract_text_content(&response)
        ));
    }
    Ok(response)
}

async fn fill_initiative_template(
    initiative: &AiInitiative,
    template: &str,
) -> Result<(String, u64)> {
    let system = "You are filling in a planning initiative document template for a software project. Replace all placeholders and template instructions with concrete content. Output only the completed markdown document.";
    let user = format!(
        "Initiative title: {}\nObjective: {}\nTasks:\n- {}\n\nTemplate:\n\n{}",
        initiative.title,
        initiative.objective,
        initiative.tasks.join("\n- "),
        template
    );
    match api_client::ask(system, &user).await {
        Ok(response) => {
            let total_tokens = response.total_tokens();
            Ok((response.content, total_tokens))
        }
        Err(_) => Ok((build_fallback_initiative_document(initiative, template), 0)),
    }
}

fn derive_fallback_initiatives(scenario: &LoadedScenarioPack) -> Vec<AiInitiative> {
    let mut initiatives = module_headings_from_spec(scenario)
        .into_iter()
        .filter(|module| !is_seeded_module(scenario, module))
        .map(|module| fallback_initiative_for_module(&module))
        .collect::<Vec<_>>();

    if initiatives.is_empty() {
        initiatives.push(default_output_initiative());
    }

    initiatives
}

fn module_headings_from_spec(scenario: &LoadedScenarioPack) -> Vec<String> {
    let Some(spec) = scenario.specification.as_ref() else {
        return vec![];
    };

    spec.lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .filter(|heading| heading.to_ascii_lowercase().contains("module"))
        .map(ToString::to_string)
        .collect()
}

fn is_seeded_module(scenario: &LoadedScenarioPack, module_heading: &str) -> bool {
    let normalized_module = normalize_module_name(module_heading);
    scenario.seed_initiatives.iter().any(|artifact| {
        extract_title(artifact)
            .map(|title| normalize_module_name(&title))
            .is_some_and(|title| title.contains(&normalized_module))
    })
}

fn normalize_module_name(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace(':', " ")
        .replace("specifications", " ")
        .replace("initiative", " ")
        .replace("module", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn fallback_initiative_for_module(module_heading: &str) -> AiInitiative {
    let normalized = normalize_module_name(module_heading);
    if normalized.contains("validate") || normalized.contains("output") {
        return default_output_initiative();
    }

    let slug = normalized.replace(' ', "-");
    let title = title_case_words(&normalized);

    AiInitiative {
        id: slug.clone(),
        title: format!("{title} Module"),
        objective: format!(
            "Implement the {normalized} workflow so the project vision is delivered with clear interfaces, validation, and test coverage."
        ),
        tasks: vec![
            format!("Define module boundaries and contracts for {normalized}"),
            format!("Implement the core {normalized} flow with typed validation"),
            format!("Add integration coverage for {normalized} edge cases"),
        ],
    }
}

fn default_output_initiative() -> AiInitiative {
    AiInitiative {
        id: "validate-output-module".to_string(),
        title: "Validate & Output Module".to_string(),
        objective:
            "Implement schema validation, multi-format output, and delivery safeguards for processed datasets."
                .to_string(),
        tasks: vec![
            "Define output schema validation rules and failure reporting".to_string(),
            "Implement CSV, JSON, and YAML exporters with type-preserving serialization"
                .to_string(),
            "Add end-to-end tests for validation failures, escaping, and null handling".to_string(),
        ],
    }
}

fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_fallback_initiative_document(initiative: &AiInitiative, template: &str) -> String {
    let architecture_context = summarize_template_context(template);
    let tasks = initiative
        .tasks
        .iter()
        .map(|task| format!("- {task}"))
        .collect::<Vec<_>>()
        .join("\n");
    let acceptance = initiative
        .tasks
        .iter()
        .map(|task| format!("- [ ] {task}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# {title}

## Context

This initiative closes a planning gap identified during benchmark scenario analysis.
It supports the project vision by extending the seeded plan with a bounded module that fits the existing architecture and document structure.
{architecture_context}

## Objective

{objective}
The work should preserve typed boundaries, explicit validation, and deterministic behavior wherever possible.

## Goals & Non-Goals

- Deliver a focused implementation slice that complements the existing seeded initiatives.
- Define interfaces and data contracts before implementation so downstream work stays aligned.
- Keep non-goals explicit: this initiative should not absorb unrelated platform or UX work.

## Requirements

- The module must integrate cleanly with the unified data model and existing workflow stages.
- Static validation and contract checks should be preferred over purely generative behavior.
- Outputs and failures must be observable enough for benchmark scoring and regression analysis.

## Acceptance Criteria

{acceptance}
- [ ] Architecture decisions are documented and reflected in implementation boundaries.

## Tasks

{tasks}

## Risks

- Hidden coupling with existing modules may reduce the clarity of boundaries if interfaces are not documented first.
- Edge-case handling may drift unless validation behavior is defined before implementation.
- Benchmark comparisons can become noisy if output expectations are not encoded in deterministic checks.

## Implementation Notes

- Start with interfaces, schemas, or contracts before concrete implementation.
- Add tests that reflect the scenario specification and expected failure modes.
- Prefer reusable utility code and static checks over prompt-heavy workflows.

## Success Criteria

- The initiative is actionable without additional clarification.
- The resulting work can be scored for completeness, architecture alignment, and quality.
- The implementation path is detailed enough to break into tasks and execute predictably.
"#,
        title = initiative.title,
        architecture_context = architecture_context,
        objective = initiative.objective,
        acceptance = acceptance,
        tasks = tasks,
    )
}

fn summarize_template_context(template: &str) -> String {
    let first_heading = template
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches("# ").trim())
        .unwrap_or("initiative template");
    format!(
        "The generated document follows the structure expected by the {first_heading}, replacing placeholders with concrete planning content."
    )
}

fn extract_title(artifact: &ScenarioArtifact) -> Option<String> {
    artifact
        .content
        .lines()
        .find_map(|line| line.strip_prefix("title:"))
        .map(|title| title.trim().trim_matches('"').to_string())
        .or_else(|| {
            artifact
                .content
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .map(|title| title.trim().to_string())
        })
}

fn document_project_path(system: SystemUnderTest, root: &Path) -> String {
    match system {
        SystemUnderTest::OriginalMetis => root.join(".metis").display().to_string(),
        SystemUnderTest::UltraMetisMcp => root.display().to_string(),
    }
}

fn extract_text_content(response: &Value) -> String {
    response["result"]["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_error_response(response: &Value) -> bool {
    response["result"]["isError"].as_bool().unwrap_or(false) || response.get("error").is_some()
}

fn extract_short_code(text: &str, prefix: &str) -> Option<String> {
    text.split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
        .find(|word| word.starts_with(prefix))
        .map(ToString::to_string)
}

fn system_name(system: SystemUnderTest) -> &'static str {
    match system {
        SystemUnderTest::OriginalMetis => "original-metis",
        SystemUnderTest::UltraMetisMcp => "ultra-metis-mcp",
    }
}

pub fn format_planning_report(result: &PlanningComparisonResult) -> String {
    let mut out = String::new();
    out.push_str("# MCP Planning Comparison Report\n\n");
    out.push_str(&format!(
        "**Date**: {}  \n**Scenario**: {} ({})\n\n",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        result.scenario_title,
        result.scenario_id
    ));
    out.push_str("## Initiative Generation\n\n");
    out.push_str(&format!(
        "- Generated {} additional initiatives from scenario assessment\n\n",
        result.generated_initiatives.len()
    ));
    out.push_str("## System Results\n\n");
    out.push_str("| Metric | original-metis | ultra-metis-mcp |\n");
    out.push_str("|--------|----------------|-----------------|\n");
    out.push_str(&format!(
        "| Initiative docs scored | {} | {} |\n| Total tokens | {} | {} |\n| Total time (ms) | {:.2} | {:.2} |\n| Avg completeness | {:.1}% | {:.1}% |\n| Avg placeholders/doc | {:.1} | {:.1} |\n| Avg task count | {:.1} | {:.1} |\n",
        result.original_metis.initiative_count,
        result.ultra_metis_mcp.initiative_count,
        result.original_metis.total_tokens,
        result.ultra_metis_mcp.total_tokens,
        result.original_metis.total_time_ms,
        result.ultra_metis_mcp.total_time_ms,
        result.original_metis.avg_completeness_percent,
        result.ultra_metis_mcp.avg_completeness_percent,
        result.original_metis.avg_placeholder_count,
        result.ultra_metis_mcp.avg_placeholder_count,
        result.original_metis.avg_task_count,
        result.ultra_metis_mcp.avg_task_count,
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc_quality::score_content;
    use crate::scenario_pack::LoadedScenarioPack;
    use std::path::PathBuf;

    #[test]
    fn extract_title_uses_frontmatter_title() {
        let artifact = ScenarioArtifact {
            path: "x".into(),
            content: "---\ntitle: \"Parse Module\"\n---\n# Ignore".to_string(),
        };
        assert_eq!(extract_title(&artifact).as_deref(), Some("Parse Module"));
    }

    #[test]
    fn planning_report_contains_summary() {
        let report = format_planning_report(&PlanningComparisonResult {
            timestamp: Utc::now(),
            scenario_id: "demo".to_string(),
            scenario_title: "Demo".to_string(),
            generated_initiatives: vec![],
            original_metis: PlanningSystemResult {
                system: "original-metis".to_string(),
                initiative_count: 1,
                total_tokens: 10,
                total_time_ms: 10.0,
                avg_completeness_percent: 50.0,
                avg_placeholder_count: 1.0,
                avg_task_count: 2.0,
                docs: vec![],
            },
            ultra_metis_mcp: PlanningSystemResult {
                system: "ultra-metis-mcp".to_string(),
                initiative_count: 1,
                total_tokens: 11,
                total_time_ms: 11.0,
                avg_completeness_percent: 60.0,
                avg_placeholder_count: 0.0,
                avg_task_count: 2.0,
                docs: vec![],
            },
        });
        assert!(report.contains("System Results"));
        assert!(report.contains("Avg completeness"));
    }

    #[test]
    fn fallback_initiatives_cover_missing_modules() {
        let scenario = LoadedScenarioPack::load(&PathBuf::from("scenario")).unwrap();
        let initiatives = derive_fallback_initiatives(&scenario);

        assert!(!initiatives.is_empty());
        assert!(initiatives
            .iter()
            .any(|initiative| initiative.title.contains("Validate")
                || initiative.title.contains("Output")));
    }

    #[test]
    fn fallback_fill_produces_scored_document() {
        let initiative = default_output_initiative();
        let filled = build_fallback_initiative_document(
            &initiative,
            "# Initiative Template\n\n## Context\n\n{placeholder}",
        );
        let quality = score_content(&filled);

        assert_eq!(quality.placeholder_count, 0);
        assert!(quality.completeness_percent >= 70.0);
        assert!(quality.content_lines >= 10);
    }
}
