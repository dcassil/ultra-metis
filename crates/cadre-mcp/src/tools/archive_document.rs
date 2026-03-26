use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "archive_document",
    description = "Archive a document and all its children using its short code (e.g., PROJ-V-0001). The document and its children will be moved to the archived folder and marked as archived.",
    idempotent_hint = false,
    destructive_hint = true,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveDocumentTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
}

impl ArchiveDocumentTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);

        // List children before archiving to show in output
        let all_docs = store
            .list_documents(false)
            .map_err(|e| tool_error(e.user_message()))?;
        let children: Vec<_> = all_docs
            .iter()
            .filter(|d| d.parent_id.as_deref() == Some(&self.short_code))
            .collect();

        store
            .archive_document(&self.short_code)
            .map_err(|e| tool_error(e.user_message()))?;

        let mut output = format!("## Archived: {}\n", self.short_code);
        if !children.is_empty() {
            output.push_str(&format!("\nAlso archived {} children:\n", children.len()));
            for child in &children {
                output.push_str(&format!("  - {} ({})\n", child.short_code, child.title));
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
