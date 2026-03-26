use super::helpers::{store_for, tool_error};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "create_document",
    description = "Create a new Cadre document (vision, strategy, initiative, task, adr, product_doc, epic, story, design_context, analysis_baseline, quality_record, rules_config, durable_insight_note, cross_reference, architecture_catalog_entry, reference_architecture). Returns the new document's short code.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocumentTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document type: vision, strategy, initiative, task, adr, product_doc, epic, story, design_context, analysis_baseline, quality_record, rules_config, durable_insight_note, cross_reference, architecture_catalog_entry, reference_architecture
    pub document_type: String,
    /// Title of the document
    pub title: String,
    /// Parent document short code (optional)
    pub parent_id: Option<String>,
}

impl CreateDocumentTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let short_code = store
            .create_document(&self.document_type, &self.title, self.parent_id.as_deref())
            .map_err(|e| tool_error(e.user_message()))?;

        let parent_row = if let Some(pid) = &self.parent_id {
            format!("\n| Parent | {pid} |")
        } else {
            String::new()
        };

        let text = format!(
            "## Document Created\n\n\
            {} created successfully\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Title | {} |\n\
            | Type | {} |\n\
            | Short Code | {} |{}",
            short_code, self.title, self.document_type, short_code, parent_row
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
