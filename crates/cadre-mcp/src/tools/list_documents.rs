use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_documents",
    description = "List documents in a project with optional filtering. Returns document details including unique short codes (format: PREFIX-TYPE-NNNN).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDocumentsTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Include archived documents in results (defaults to false)
    #[serde(default)]
    pub include_archived: Option<bool>,
    /// Filter to only show children of this parent document (optional)
    pub parent_id: Option<String>,
}

impl ListDocumentsTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let include_archived = self.include_archived.unwrap_or(false);
        let docs = store
            .list_documents_with_options(include_archived, self.parent_id.as_deref())
            .map_err(|e| tool_error(e.user_message()))?;

        if docs.is_empty() {
            let text = "No documents found.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Documents ({})\n\n\
            | Short Code | Title | Type | Phase | Parent |\n\
            | ---------- | ----- | ---- | ----- | ------ |\n",
            docs.len()
        );
        for doc in &docs {
            let parent = doc.parent_id.as_deref().unwrap_or("-");
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                doc.short_code, doc.title, doc.document_type, doc.phase, parent
            ));
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(output, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
