use super::helpers::{build_traceability_index, capitalize_first, store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "trace_ancestry",
    description = "Walk the document hierarchy to find ancestors, descendants, or siblings.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TraceAncestryTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document short code to trace from
    pub short_code: String,
    /// Direction: ancestors, descendants, or siblings
    pub direction: String,
}

impl TraceAncestryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let (index, _) = build_traceability_index(&store)?;

        let results = match self.direction.as_str() {
            "ancestors" => index.ancestors(&self.short_code),
            "descendants" => index.descendants(&self.short_code),
            "siblings" => index.siblings(&self.short_code),
            _ => {
                return Err(tool_error(format!(
                    "Invalid direction '{}'. Use: ancestors, descendants, or siblings",
                    self.direction
                )))
            }
        };

        if results.is_empty() {
            let text = format!(
                "No {} found for {}.",
                self.direction, self.short_code
            );
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## {} of {} ({})\n\n",
            capitalize_first(&self.direction),
            self.short_code,
            results.len()
        );
        for (i, sc) in results.iter().enumerate() {
            let title = store
                .read_document(sc)
                .map(|d| d.title().to_string())
                .unwrap_or_else(|_| "?".to_string());
            let prefix = if self.direction == "ancestors" {
                "  ".repeat(i)
            } else {
                String::new()
            };
            output.push_str(&format!("{}- {} ({})\n", prefix, sc, title));
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(output, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
