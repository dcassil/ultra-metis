use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_quality_records",
    description = "List quality records with optional status filtering (pass/warn/fail).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListQualityRecordsTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Filter by status: pass, warn, or fail (optional)
    pub status: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<u64>,
}

impl ListQualityRecordsTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let docs = store
            .search_documents_with_options("quality_record", Some("quality_record"), None, false)
            .map_err(|e| tool_error(e.user_message()))?;

        let baselines = store
            .search_documents_with_options(
                "analysis_baseline",
                Some("analysis_baseline"),
                None,
                false,
            )
            .map_err(|e| tool_error(e.user_message()))?;

        let mut results = Vec::new();
        for doc in &docs {
            if let Some(filter) = &self.status {
                let raw = store.read_document_raw(&doc.short_code).unwrap_or_default();
                if !raw
                    .to_lowercase()
                    .contains(&format!("overall_status: {}", filter.to_lowercase()))
                {
                    continue;
                }
            }
            results.push(doc);
        }

        if let Some(lim) = self.limit {
            results.truncate(lim as usize);
        }

        let mut output = format!(
            "## Quality Records ({})\n\n\
            | Short Code | Title | Phase |\n\
            | ---------- | ----- | ----- |\n",
            results.len()
        );
        for doc in &results {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                doc.short_code, doc.title, doc.phase
            ));
        }

        if !baselines.is_empty() {
            output.push_str(&format!(
                "\n## Analysis Baselines ({})\n\n\
                | Short Code | Title | Phase |\n\
                | ---------- | ----- | ----- |\n",
                baselines.len()
            ));
            for doc in &baselines {
                output.push_str(&format!(
                    "| {} | {} | {} |\n",
                    doc.short_code, doc.title, doc.phase
                ));
            }
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(output, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
