use super::helpers::{store_for, tool_error};
use cadre_core::{CrossReference, Phase, RelationshipType, Tag};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "create_cross_reference",
    description = "Create a typed relationship between two documents.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateCrossReferenceTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Short code of the source document
    pub source_ref: String,
    /// Short code of the target document
    pub target_ref: String,
    /// Type: parent_child, governs, references, derived_from, supersedes, conflicts_with, validates, blocks, approved_by
    pub relationship_type: String,
    /// Human-readable description of the relationship (optional)
    pub description: Option<String>,
    /// Whether traversable in both directions (default: false)
    #[serde(default)]
    pub bidirectional: Option<bool>,
}

impl CreateCrossReferenceTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        if self.source_ref == self.target_ref {
            return Err(tool_error("Source and target cannot be the same document."));
        }

        let rel_type: RelationshipType = self
            .relationship_type
            .parse()
            .map_err(|e: String| tool_error(e))?;
        let bidirectional = self.bidirectional.unwrap_or(false);
        let description = self.description.as_deref().unwrap_or("");

        let store = store_for(&self.project_path);

        // Verify both documents exist
        store
            .read_document(&self.source_ref)
            .map_err(|e| tool_error(e.user_message()))?;
        store
            .read_document(&self.target_ref)
            .map_err(|e| tool_error(e.user_message()))?;

        let title = format!("{} {} {}", self.source_ref, rel_type, self.target_ref);
        let short_code = store
            .create_document("cross_reference", &title, None)
            .map_err(|e| tool_error(e.user_message()))?;

        let xref = CrossReference::new(
            title.clone(),
            vec![Tag::Phase(Phase::Draft)],
            false,
            short_code.clone(),
            self.source_ref.clone(),
            self.target_ref.clone(),
            rel_type,
            description.to_string(),
            bidirectional,
        )
        .map_err(|e| tool_error(e))?;

        let content = xref.to_content().map_err(|e| tool_error(e))?;
        let doc_path = Path::new(&self.project_path)
            .join(".cadre")
            .join("docs")
            .join(format!("{}.md", short_code));
        std::fs::write(&doc_path, content).map_err(|e| tool_error(e))?;

        let bidir_label = if bidirectional {
            " (bidirectional)"
        } else {
            ""
        };
        let text = format!(
            "## Cross-Reference Created{}\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Short Code | {} |\n\
            | Source | {} |\n\
            | Target | {} |\n\
            | Type | {} |\n\
            | Description | {} |",
            bidir_label, short_code, self.source_ref, self.target_ref, rel_type, description
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
