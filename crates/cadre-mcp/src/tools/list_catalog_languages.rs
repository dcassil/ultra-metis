use cadre_core::CatalogQueryEngine;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_catalog_languages",
    description = "List all available languages and project types in the architecture catalog.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListCatalogLanguagesTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
}

impl ListCatalogLanguagesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let engine = CatalogQueryEngine::with_builtins();
        let languages = engine.languages();

        if languages.is_empty() {
            let text = "No languages found in the catalog.".to_string();
            return Ok(CallToolResult {
                content: vec![TextContent::new(text, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        let mut output = "## Architecture Catalog Languages\n\n".to_string();
        for lang in &languages {
            let types = engine.project_types_for_language(lang);
            output.push_str(&format!("### {}\n\n", lang));
            for pt in &types {
                output.push_str(&format!("- {}\n", pt));
            }
            output.push('\n');
        }

        Ok(CallToolResult {
            content: vec![TextContent::new(output, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
