use super::helpers::{project_root_from, store_for, tool_error};
use cadre_core::{
    BaselineCaptureService, ClippyParser, CoverageParser, EslintParser, ToolOutputParser,
    TypeScriptParser,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "capture_quality_baseline",
    description = "Parse raw tool output (eslint/clippy/tsc/coverage) and create an AnalysisBaseline document.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaptureQualityBaselineTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Tool name: eslint, clippy, tsc, or coverage
    pub tool_name: String,
    /// Raw tool output string to parse
    pub raw_output: String,
    /// Short code of linked RulesConfig (optional)
    pub linked_rules_config: Option<String>,
}

impl CaptureQualityBaselineTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let parsed = match self.tool_name.to_lowercase().as_str() {
            "eslint" => EslintParser
                .parse(&self.raw_output)
                .map_err(tool_error)?,
            "clippy" => ClippyParser
                .parse(&self.raw_output)
                .map_err(tool_error)?,
            "tsc" | "typescript" => TypeScriptParser
                .parse(&self.raw_output)
                .map_err(tool_error)?,
            "coverage" => CoverageParser
                .parse(&self.raw_output)
                .map_err(tool_error)?,
            _ => {
                return Err(tool_error(format!(
                    "Unknown tool: {}. Supported: eslint, clippy, tsc, coverage",
                    self.tool_name
                )))
            }
        };

        let store = store_for(&self.project_path);
        let short_code = store
            .create_document(
                "analysis_baseline",
                &format!("{} Baseline", self.tool_name),
                None,
            )
            .map_err(|e| tool_error(e.user_message()))?;

        let baseline =
            BaselineCaptureService::capture(&parsed, &short_code, self.linked_rules_config.clone())
                .map_err(tool_error)?;

        let content = baseline.to_content().map_err(tool_error)?;
        let project_root = project_root_from(&self.project_path);
        let doc_path = project_root
            .join(".cadre")
            .join("docs")
            .join(format!("{short_code}.md"));
        std::fs::write(&doc_path, content).map_err(tool_error)?;

        let text = format!(
            "## Quality Baseline Captured\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Short Code | {} |\n\
            | Tool | {} |\n\
            | Total Findings | {} |\n\
            | Errors | {} |\n\
            | Warnings | {} |\n\
            | Metrics | {} |",
            short_code,
            parsed.tool_name,
            parsed.total_findings(),
            parsed.error_count(),
            parsed.warning_count(),
            parsed.metrics.len(),
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
