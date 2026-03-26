use super::helpers::{extract_tool_from_baseline, store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "compare_quality_baselines",
    description = "Compare two AnalysisBaseline documents and produce a QualityRecord with metric deltas.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CompareQualityBaselinesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Short code of the 'before' AnalysisBaseline
    pub before_short_code: String,
    /// Short code of the 'after' AnalysisBaseline
    pub after_short_code: String,
}

impl CompareQualityBaselinesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);

        let before_raw = store
            .read_document_raw(&self.before_short_code)
            .map_err(|e| tool_error(e.user_message()))?;
        let after_raw = store
            .read_document_raw(&self.after_short_code)
            .map_err(|e| tool_error(e.user_message()))?;

        let before_tool = extract_tool_from_baseline(&before_raw)
            .ok_or_else(|| tool_error("Could not determine tool from 'before' baseline"))?;
        let after_tool = extract_tool_from_baseline(&after_raw)
            .ok_or_else(|| tool_error("Could not determine tool from 'after' baseline"))?;

        if before_tool != after_tool {
            return Err(tool_error(format!(
                "Cannot compare baselines from different tools: '{before_tool}' vs '{after_tool}'"
            )));
        }

        let qr_code = store
            .create_document(
                "quality_record",
                &format!(
                    "{} Comparison: {} vs {}",
                    before_tool, self.before_short_code, self.after_short_code
                ),
                None,
            )
            .map_err(|e| tool_error(e.user_message()))?;

        let text = format!(
            "## Quality Comparison\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Before | {} |\n\
            | After | {} |\n\
            | Tool | {} |\n\
            | Record | {} |\n\n\
            Comparison record created. Edit {} to add detailed metric deltas and findings.",
            self.before_short_code, self.after_short_code, before_tool, qr_code, qr_code
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
