use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "read_document",
    description = "Read a document's content and structure using its short code (e.g., PROJ-V-0001).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadDocumentTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001)
    pub short_code: String,
}

impl ReadDocumentTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let raw = store
            .read_document_raw(&self.short_code)
            .map_err(|e| tool_error(e.user_message()))?;

        Ok(CallToolResult {
            content: vec![TextContent::new(raw, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
