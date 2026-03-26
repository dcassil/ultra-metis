use cadre_core::{CatalogQuery, CatalogQueryEngine};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "query_architecture_catalog",
    description = "Search the architecture catalog by language and project type.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryArchitectureCatalogTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Filter by language (optional)
    pub language: Option<String>,
    /// Filter by project type (optional)
    pub project_type: Option<String>,
}

impl QueryArchitectureCatalogTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let engine = CatalogQueryEngine::with_builtins();

        let mut query = CatalogQuery::new();
        if let Some(lang) = &self.language {
            query = query.with_language(lang);
        }
        if let Some(pt) = &self.project_type {
            query = query.with_project_type(pt);
        }

        let matches = engine.query(&query);

        if matches.is_empty() {
            let filter_desc = match (&self.language, &self.project_type) {
                (Some(l), Some(p)) => format!("language='{}', project_type='{}'", l, p),
                (Some(l), None) => format!("language='{}'", l),
                (None, Some(p)) => format!("project_type='{}'", p),
                (None, None) => "no filters".to_string(),
            };
            let text = format!("No catalog entries found for {}.", filter_desc);
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = format!(
            "## Architecture Catalog ({} entries)\n\n",
            matches.len()
        );
        for m in &matches {
            let entry = m.entry;
            output.push_str(&format!(
                "### {} — {} / {}\n\n",
                entry.title(),
                entry.language,
                entry.project_type
            ));
            if !entry.folder_layout.is_empty() {
                output.push_str(&format!(
                    "**Folder Layout**: {}\n\n",
                    entry.folder_layout.join(", ")
                ));
            }
            if !entry.layers.is_empty() {
                output.push_str(&format!("**Layers**: {}\n\n", entry.layers.join(", ")));
            }
            if !entry.dependency_rules.is_empty() {
                output.push_str(&format!(
                    "**Dependency Rules**: {}\n\n",
                    entry.dependency_rules.join("; ")
                ));
            }
            if !entry.naming_conventions.is_empty() {
                output.push_str(&format!(
                    "**Naming Conventions**: {}\n\n",
                    entry.naming_conventions.join("; ")
                ));
            }
            output.push_str("---\n\n");
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(output, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
