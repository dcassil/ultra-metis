use super::helpers::tool_error;
use cadre_store::CodeIndexer;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "index_code",
    description = "Index source code symbols using tree-sitter for cross-referencing with documents. Query by name/kind.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexCodeTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Glob patterns for source files to index (e.g., ["src/**/*.rs"])
    pub patterns: Option<Vec<String>>,
    /// Search query to find symbols by name (optional, indexes if omitted)
    pub query: Option<String>,
    /// Filter symbols by kind: function, struct, trait, enum, impl, type_alias, const, static, mod
    pub kind: Option<String>,
}

impl IndexCodeTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let index_path = Path::new(&self.project_path)
            .join(".cadre")
            .join("code-index.json");

        // If query is provided, search existing index
        if let Some(q) = &self.query {
            let index_content = std::fs::read_to_string(&index_path).map_err(|_| {
                tool_error(
                    "No code index found. Run index_code without a query first to build the index.",
                )
            })?;
            let index: cadre_store::CodeIndex =
                serde_json::from_str(&index_content).map_err(|e| tool_error(e))?;

            let results =
                CodeIndexer::search_symbols(&index, Some(q), self.kind.as_deref());

            if results.is_empty() {
                let text = format!("No symbols matching '{}'", q);
                return Ok(CallToolResult {
                    content: vec![TextContent::new(text, None, None).into()],
                    is_error: None,
                    meta: None,
                    structured_content: None,
                });
            }

            let mut output = format!(
                "## Symbol Search: '{}'\n\n\
                | Name | Kind | File | Line | Signature |\n\
                | ---- | ---- | ---- | ---- | --------- |\n",
                q
            );
            for sym in &results {
                output.push_str(&format!(
                    "| {} | {} | {} | {} | `{}` |\n",
                    sym.name, sym.kind, sym.file_path, sym.line_number, sym.signature
                ));
            }
            return Ok(CallToolResult {
                content: vec![TextContent::new(output, None, None).into()],
                is_error: None,
                meta: None,
                structured_content: None,
            });
        }

        // Otherwise, build the index
        let patterns = self
            .patterns
            .clone()
            .unwrap_or_else(|| vec!["src/**/*.rs".to_string()]);

        let indexer = CodeIndexer::new(Path::new(&self.project_path));
        let index = indexer.index(&patterns).map_err(|e| tool_error(e))?;

        let symbol_count = index.symbols.len();
        let file_count = index.indexed_files;

        // Save index to disk
        let json = serde_json::to_string_pretty(&index).map_err(|e| tool_error(e))?;
        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| tool_error(e))?;
        }
        std::fs::write(&index_path, json).map_err(|e| tool_error(e))?;

        let text = format!(
            "## Code Index Built\n\nIndexed {} symbols across {} files.\nIndex saved to: {}",
            symbol_count,
            file_count,
            index_path.display()
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
