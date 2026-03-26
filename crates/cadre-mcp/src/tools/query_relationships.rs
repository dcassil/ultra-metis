use super::helpers::{build_traceability_index, store_for, tool_error};
use cadre_core::RelationshipType;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "query_relationships",
    description = "Query all relationships involving a specific document.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryRelationshipsTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Document short code to query relationships for
    pub short_code: String,
    /// Direction: outgoing, incoming, or all (default: all)
    pub direction: Option<String>,
    /// Filter by relationship type (optional)
    pub relationship_type: Option<String>,
}

impl QueryRelationshipsTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let direction = self.direction.as_deref().unwrap_or("all");

        let store = store_for(&self.project_path);
        let (index, _) = build_traceability_index(&store)?;

        let entries = match direction {
            "outgoing" => index.outgoing(&self.short_code),
            "incoming" => index.incoming(&self.short_code),
            _ => index.involving(&self.short_code),
        };

        let filtered: Vec<_> = if let Some(rt) = &self.relationship_type {
            let rel_type: RelationshipType = rt.parse().map_err(|e: String| tool_error(e))?;
            entries
                .into_iter()
                .filter(|e| e.relationship_type == rel_type)
                .collect()
        } else {
            entries
        };

        if filtered.is_empty() {
            let text = format!(
                "No {} relationships found for {}.",
                direction, self.short_code
            );
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Relationships for {} ({}, {})\n\n\
            | Source | Type | Target | Bidirectional |\n\
            | ------ | ---- | ------ | ------------- |\n",
            self.short_code,
            direction,
            filtered.len()
        );
        for entry in &filtered {
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                entry.source_ref, entry.relationship_type, entry.target_ref, entry.bidirectional
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
