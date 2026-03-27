use super::helpers::{cadre_internal_dir, project_root_from, tool_error};
use cadre_store::CodeIndexer;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

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
        let project_root = project_root_from(&self.project_path);
        let internal_dir = cadre_internal_dir(&self.project_path);
        let index_path = internal_dir.join("code-index.json");

        // If query is provided, search existing index
        if let Some(q) = &self.query {
            let index_content = std::fs::read_to_string(&index_path).map_err(|_| {
                tool_error(
                    "No code index found. Run index_code without a query first to build the index.",
                )
            })?;
            let index: cadre_store::CodeIndex =
                serde_json::from_str(&index_content).map_err(tool_error)?;

            let results = CodeIndexer::search_symbols(&index, Some(q), self.kind.as_deref());

            if results.is_empty() {
                let text = format!("No symbols matching '{q}'");
                return Ok(CallToolResult {
                    content: vec![TextContent::new(text, None, None).into()],
                    is_error: None,
                    meta: None,
                    structured_content: None,
                });
            }

            let mut output = format!(
                "## Symbol Search: '{q}'\n\n\
                | Name | Kind | File | Line | Signature |\n\
                | ---- | ---- | ---- | ---- | --------- |\n"
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

        // Resolve source directory: use cached source_dir from index, or try src/, or ask agent
        let source_dir = self.resolve_source_dir(&project_root, &index_path)?;

        // Build patterns relative to project root using resolved source dir
        let patterns = self.patterns.clone().unwrap_or_else(|| {
            let rel_source = source_dir
                .strip_prefix(&project_root)
                .unwrap_or(&source_dir);
            vec![format!("{}/**/*.rs", rel_source.display())]
        });

        let indexer = CodeIndexer::new(&project_root);
        let mut index = indexer.index(&patterns).map_err(tool_error)?;

        // Store the resolved source_dir in the index for future calls
        let rel_source = source_dir
            .strip_prefix(&project_root)
            .unwrap_or(&source_dir);
        index.source_dir = Some(rel_source.display().to_string());

        let symbol_count = index.symbols.len();
        let file_count = index.indexed_files;

        // Save index to disk
        let json = serde_json::to_string_pretty(&index).map_err(tool_error)?;
        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent).map_err(tool_error)?;
        }
        std::fs::write(&index_path, json).map_err(tool_error)?;

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

    /// Resolve the source directory for indexing.
    /// Priority: 1) cached source_dir from existing index, 2) src/ if it exists, 3) error asking agent
    fn resolve_source_dir(
        &self,
        project_root: &std::path::Path,
        index_path: &std::path::Path,
    ) -> Result<std::path::PathBuf, CallToolError> {
        // If patterns were explicitly provided, skip resolution
        if self.patterns.is_some() {
            return Ok(project_root.to_path_buf());
        }

        // Check existing index for cached source_dir
        if index_path.exists() {
            if let Ok(content) = std::fs::read_to_string(index_path) {
                if let Ok(existing_index) =
                    serde_json::from_str::<cadre_store::CodeIndex>(&content)
                {
                    if let Some(ref cached_dir) = existing_index.source_dir {
                        let resolved = project_root.join(cached_dir);
                        if resolved.is_dir() {
                            return Ok(resolved);
                        }
                    }
                }
            }
        }

        // Try src/ relative to project root
        let src_dir = project_root.join("src");
        if src_dir.is_dir() {
            return Ok(src_dir);
        }

        // No source directory found — ask the agent for help
        Err(tool_error(format!(
            "Could not find source directory. No `src/` directory found at `{}`. \
            Please provide explicit `patterns` parameter with glob patterns pointing to your source files \
            (e.g., [\"lib/**/*.rs\"] or [\"app/**/*.ts\"]). \
            The resolved path will be cached in code-index.json for future calls.",
            project_root.display()
        )))
    }
}
