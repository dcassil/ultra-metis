use crate::mcp_adapter::{
    expected_shared_tool_names, ExecutionAdapter, McpSession, OriginalMetisAdapter,
    SystemUnderTest, CadreMcpAdapter,
};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOperationResult {
    pub operation: String,
    pub ok: bool,
    pub duration_ms: f64,
    pub output_size: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRun {
    pub system: String,
    pub tool_surface: String,
    pub startup_ok: bool,
    pub tools_available: Vec<String>,
    pub operations: Vec<McpOperationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpComparisonResult {
    pub timestamp: chrono::DateTime<Utc>,
    pub scenario_id: String,
    pub scenario_title: String,
    pub original_metis: McpToolRun,
    pub cadre_mcp: McpToolRun,
}

pub fn run_shared_tool_comparison(
    scenario_id: &str,
    scenario_title: &str,
) -> Result<McpComparisonResult> {
    let ultra_dir = tempfile::tempdir().context("Failed to create ultra tempdir")?;
    let orig_dir = tempfile::tempdir().context("Failed to create original tempdir")?;

    let ultra = run_tool_workflow(
        &CadreMcpAdapter,
        ultra_dir.path(),
        "BENCH",
        "File Processing Toolkit",
        "Parse Module",
        "Parser",
    )?;
    let original = run_tool_workflow(
        &OriginalMetisAdapter,
        orig_dir.path(),
        "BENCH",
        "File Processing Toolkit",
        "Parse Module",
        "Parse",
    )?;

    Ok(McpComparisonResult {
        timestamp: Utc::now(),
        scenario_id: scenario_id.to_string(),
        scenario_title: scenario_title.to_string(),
        original_metis: original,
        cadre_mcp: ultra,
    })
}

fn run_tool_workflow<A: ExecutionAdapter>(
    adapter: &A,
    project_path: &Path,
    prefix: &str,
    vision_title: &str,
    initiative_title: &str,
    search_term: &str,
) -> Result<McpToolRun> {
    let mut session = adapter.start()?;
    let mut operations = vec![];

    let tools = session.list_tools().context("Failed to list MCP tools")?;
    let tool_names = tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
    for expected in expected_shared_tool_names() {
        if !tool_names.iter().any(|name| name == expected) {
            anyhow::bail!(
                "Adapter {:?} missing expected tool '{}'",
                adapter.system_under_test(),
                expected
            );
        }
    }

    let root_project = project_path.display().to_string();

    record_call(
        &mut session,
        &mut operations,
        "initialize_project",
        json!({
            "project_path": root_project,
            "prefix": prefix,
        }),
    )?;

    let document_project_path = document_project_path(adapter.system_under_test(), project_path);

    let vision_code = match adapter.system_under_test() {
        SystemUnderTest::OriginalMetis => {
            operations.push(McpOperationResult {
                operation: "create_vision".to_string(),
                ok: true,
                duration_ms: 0.0,
                output_size: 28,
                detail: "Vision auto-created during init".to_string(),
            });
            "BENCH-V-0001".to_string()
        }
        SystemUnderTest::CadreMcp => {
            record_call(
                &mut session,
                &mut operations,
                "create_vision",
                json!({
                    "project_path": document_project_path,
                    "document_type": "vision",
                    "title": vision_title,
                }),
            )?;
            let detail = operations
                .iter()
                .find(|op| op.operation == "create_vision")
                .map(|op| op.detail.as_str())
                .unwrap_or("");
            extract_short_code(detail, "BENCH-V-").unwrap_or_else(|| "BENCH-V-0001".to_string())
        }
    };

    record_call(
        &mut session,
        &mut operations,
        "create_initiative",
        json!({
            "project_path": document_project_path,
            "document_type": "initiative",
            "title": initiative_title,
            "parent_id": vision_code,
        }),
    )?;

    let initiative_detail = operations
        .iter()
        .find(|op| op.operation == "create_initiative")
        .map(|op| op.detail.as_str())
        .unwrap_or("");
    let initiative_code = extract_short_code(initiative_detail, "BENCH-I-")
        .unwrap_or_else(|| "BENCH-I-0001".to_string());

    record_call(
        &mut session,
        &mut operations,
        "list_documents",
        json!({
            "project_path": document_project_path,
        }),
    )?;

    record_call(
        &mut session,
        &mut operations,
        "read_document",
        json!({
            "project_path": document_project_path,
            "short_code": initiative_code,
        }),
    )?;

    record_call(
        &mut session,
        &mut operations,
        "search_documents",
        json!({
            "project_path": document_project_path,
            "query": search_term,
        }),
    )?;

    Ok(McpToolRun {
        system: system_name(&adapter.system_under_test()).to_string(),
        tool_surface: "mcp-stdio".to_string(),
        startup_ok: true,
        tools_available: tool_names,
        operations,
    })
}

fn document_project_path(system: SystemUnderTest, root: &Path) -> String {
    match system {
        SystemUnderTest::OriginalMetis => root.join(".metis").display().to_string(),
        SystemUnderTest::CadreMcp => root.display().to_string(),
    }
}

fn record_call(
    session: &mut McpSession,
    operations: &mut Vec<McpOperationResult>,
    operation: &str,
    args: Value,
) -> Result<()> {
    let start = Instant::now();
    let response = session.call_tool(operation_tool_name(operation), args)?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let content = extract_text_content(&response);
    let ok = !is_error_response(&response);

    operations.push(McpOperationResult {
        operation: operation.to_string(),
        ok,
        duration_ms: elapsed,
        output_size: content.len(),
        detail: content,
    });

    Ok(())
}

fn operation_tool_name(operation: &str) -> &str {
    match operation {
        "initialize_project" => "initialize_project",
        "create_vision" | "create_initiative" => "create_document",
        "list_documents" => "list_documents",
        "read_document" => "read_document",
        "search_documents" => "search_documents",
        _ => operation,
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

fn system_name(system: &SystemUnderTest) -> &'static str {
    match system {
        SystemUnderTest::OriginalMetis => "original-metis",
        SystemUnderTest::CadreMcp => "cadre-mcp",
    }
}

pub fn format_comparison_report(result: &McpComparisonResult) -> String {
    let mut out = String::new();
    out.push_str("# MCP Comparison Report\n\n");
    out.push_str(&format!(
        "**Date**: {}  \n**Scenario**: {} ({})\n\n",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        result.scenario_title,
        result.scenario_id
    ));

    out.push_str("## Shared MCP Workflow\n\n");
    out.push_str("| Operation | original-metis (ms) | cadre-mcp (ms) | Winner |\n");
    out.push_str("|-----------|---------------------:|----------------------:|--------|\n");

    for (orig, ultra) in result
        .original_metis
        .operations
        .iter()
        .zip(result.cadre_mcp.operations.iter())
    {
        let winner = if orig.duration_ms < ultra.duration_ms {
            "original-metis"
        } else if ultra.duration_ms < orig.duration_ms {
            "cadre-mcp"
        } else {
            "tie"
        };
        out.push_str(&format!(
            "| {} | {:.2} | {:.2} | {} |\n",
            orig.operation, orig.duration_ms, ultra.duration_ms, winner
        ));
    }

    out.push_str("\n## Tool Surface\n\n");
    out.push_str(&format!(
        "- original-metis shared tools: {}\n",
        result.original_metis.tools_available.join(", ")
    ));
    out.push_str(&format!(
        "- cadre-mcp shared tools: {}\n",
        result.cadre_mcp.tools_available.join(", ")
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_short_code_finds_code() {
        assert_eq!(
            extract_short_code("Created BENCH-I-0001 successfully", "BENCH-I-"),
            Some("BENCH-I-0001".to_string())
        );
    }

    #[test]
    fn report_contains_operation_table() {
        let result = McpComparisonResult {
            timestamp: Utc::now(),
            scenario_id: "demo".to_string(),
            scenario_title: "Demo".to_string(),
            original_metis: McpToolRun {
                system: "original-metis".to_string(),
                tool_surface: "mcp-stdio".to_string(),
                startup_ok: true,
                tools_available: vec!["initialize_project".to_string()],
                operations: vec![McpOperationResult {
                    operation: "initialize_project".to_string(),
                    ok: true,
                    duration_ms: 10.0,
                    output_size: 10,
                    detail: "ok".to_string(),
                }],
            },
            cadre_mcp: McpToolRun {
                system: "cadre-mcp".to_string(),
                tool_surface: "mcp-stdio".to_string(),
                startup_ok: true,
                tools_available: vec!["initialize_project".to_string()],
                operations: vec![McpOperationResult {
                    operation: "initialize_project".to_string(),
                    ok: true,
                    duration_ms: 5.0,
                    output_size: 10,
                    detail: "ok".to_string(),
                }],
            },
        };

        let report = format_comparison_report(&result);
        assert!(report.contains("Shared MCP Workflow"));
        assert!(report.contains("initialize_project"));
    }

    #[test]
    fn original_metis_uses_metis_subdirectory_for_document_calls() {
        let root = Path::new("/tmp/example-project");
        assert_eq!(
            document_project_path(SystemUnderTest::OriginalMetis, root),
            "/tmp/example-project/.metis"
        );
    }

    #[test]
    fn cadre_uses_root_for_document_calls() {
        let root = Path::new("/tmp/example-project");
        assert_eq!(
            document_project_path(SystemUnderTest::CadreMcp, root),
            "/tmp/example-project"
        );
    }
}
