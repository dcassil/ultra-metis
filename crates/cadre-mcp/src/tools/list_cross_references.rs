use super::helpers::{build_traceability_index, store_for};
use cadre_core::RelationshipType;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_cross_references",
    description = "List all cross-references with optional filtering.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListCrossReferencesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Filter by relationship type (optional)
    pub relationship_type: Option<String>,
    /// Filter to show only references involving this short code (optional)
    pub involving: Option<String>,
}

impl ListCrossReferencesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let (_, xrefs) = build_traceability_index(&store)?;

        let filtered: Vec<_> = xrefs
            .iter()
            .filter(|(_, xref)| {
                if let Some(rt) = &self.relationship_type {
                    if let Ok(rel_type) = rt.parse::<RelationshipType>() {
                        if xref.relationship_type != rel_type {
                            return false;
                        }
                    }
                }
                if let Some(inv) = &self.involving {
                    if !xref.involves(inv) {
                        return false;
                    }
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            let text = "No cross-references found.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Cross-References ({})\n\n\
            | Short Code | Source | Type | Target | Bidirectional |\n\
            | ---------- | ------ | ---- | ------ | ------------- |\n",
            filtered.len()
        );
        for (sc, xref) in &filtered {
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                sc, xref.source_ref, xref.relationship_type, xref.target_ref, xref.bidirectional
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
