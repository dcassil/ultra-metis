use crate::mcp_comparison::{self, McpComparisonResult};
use crate::mcp_planning_comparison::{self, PlanningComparisonResult};
use crate::scenario_pack::LoadedScenarioPack;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSuiteResult {
    pub timestamp: chrono::DateTime<Utc>,
    pub scenario_id: String,
    pub scenario_title: String,
    pub shared_tool_comparison: McpComparisonResult,
    pub planning_comparison: SuiteStage<PlanningComparisonResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SuiteStage<T> {
    Completed { result: T },
    Failed { error: String },
}

pub async fn run_suite(scenario: &LoadedScenarioPack) -> Result<McpSuiteResult> {
    let shared_tool_comparison = mcp_comparison::run_shared_tool_comparison(
        &scenario.manifest.id,
        &scenario.manifest.title,
    )?;

    let planning_comparison = match mcp_planning_comparison::run_planning_comparison(scenario).await
    {
        Ok(result) => SuiteStage::Completed { result },
        Err(err) => SuiteStage::Failed {
            error: err.to_string(),
        },
    };

    Ok(McpSuiteResult {
        timestamp: Utc::now(),
        scenario_id: scenario.manifest.id.clone(),
        scenario_title: scenario.manifest.title.clone(),
        shared_tool_comparison,
        planning_comparison,
    })
}

pub fn format_suite_report(result: &McpSuiteResult) -> String {
    let mut out = String::new();
    out.push_str("# MCP Benchmark Suite Report\n\n");
    out.push_str(&format!(
        "**Date**: {}  \n**Scenario**: {} ({})\n\n",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        result.scenario_title,
        result.scenario_id
    ));

    out.push_str("## Stages\n\n");
    out.push_str("- Shared MCP workflow comparison: completed\n");
    match &result.planning_comparison {
        SuiteStage::Completed { .. } => {
            out.push_str("- Planning comparison: completed\n");
        }
        SuiteStage::Failed { error } => {
            out.push_str(&format!(
                "- Planning comparison: failed or skipped ({})\n",
                error
            ));
        }
    }
    out.push('\n');

    out.push_str("## Shared MCP Workflow Summary\n\n");
    out.push_str(&mcp_comparison::format_comparison_report(
        &result.shared_tool_comparison,
    ));
    out.push('\n');

    out.push_str("## Planning Comparison Summary\n\n");
    match &result.planning_comparison {
        SuiteStage::Completed { result } => {
            out.push_str(&mcp_planning_comparison::format_planning_report(result));
        }
        SuiteStage::Failed { error } => {
            out.push_str("Planning comparison did not complete.\n\n");
            out.push_str(&format!("Error: {}\n", error));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_comparison::{McpOperationResult, McpToolRun};

    #[test]
    fn suite_report_mentions_failed_planning_stage() {
        let report = format_suite_report(&McpSuiteResult {
            timestamp: Utc::now(),
            scenario_id: "demo".to_string(),
            scenario_title: "Demo".to_string(),
            shared_tool_comparison: McpComparisonResult {
                timestamp: Utc::now(),
                scenario_id: "demo".to_string(),
                scenario_title: "Demo".to_string(),
                original_metis: McpToolRun {
                    system: "original-metis".to_string(),
                    tool_surface: "mcp-stdio".to_string(),
                    startup_ok: true,
                    tools_available: vec![],
                    operations: vec![McpOperationResult {
                        operation: "initialize_project".to_string(),
                        ok: true,
                        duration_ms: 1.0,
                        output_size: 1,
                        detail: "ok".to_string(),
                    }],
                },
                ultra_metis_mcp: McpToolRun {
                    system: "ultra-metis-mcp".to_string(),
                    tool_surface: "mcp-stdio".to_string(),
                    startup_ok: true,
                    tools_available: vec![],
                    operations: vec![McpOperationResult {
                        operation: "initialize_project".to_string(),
                        ok: true,
                        duration_ms: 1.0,
                        output_size: 1,
                        detail: "ok".to_string(),
                    }],
                },
            },
            planning_comparison: SuiteStage::Failed {
                error: "no claude".to_string(),
            },
        });

        assert!(report.contains("Planning comparison did not complete"));
        assert!(report.contains("no claude"));
    }
}
