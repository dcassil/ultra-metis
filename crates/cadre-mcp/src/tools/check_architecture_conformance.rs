use super::helpers::{store_for, tool_error};
use cadre_core::DocumentType;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "check_architecture_conformance",
    description = "Run conformance check against a reference architecture using actual file paths.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CheckArchitectureConformanceTool {
    /// Path to the .cadre folder (e.g., "/Users/me/my-project/.cadre"). Must end with .cadre
    pub project_path: String,
    /// Short code of the ReferenceArchitecture document
    pub reference_arch_short_code: String,
    /// Glob patterns for files to check (e.g., ["src/**/*.rs"])
    pub file_patterns: Option<Vec<String>>,
}

impl CheckArchitectureConformanceTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let store = store_for(&self.project_path);
        let doc = store
            .read_document(&self.reference_arch_short_code)
            .map_err(|e| tool_error(e.user_message()))?;

        if doc.document_type() != DocumentType::ReferenceArchitecture {
            return Err(tool_error(format!(
                "'{}' is a {}, not a reference_architecture",
                self.reference_arch_short_code,
                doc.document_type()
            )));
        }

        let patterns = self
            .file_patterns
            .clone()
            .unwrap_or_else(|| vec!["src/**/*.rs".to_string()]);

        let mut actual_paths = Vec::new();
        let project = Path::new(&self.project_path);
        for pattern in &patterns {
            let full_pattern = project.join(pattern).display().to_string();
            if let Ok(entries) = glob::glob(&full_pattern) {
                for entry in entries.flatten() {
                    if let Ok(relative) = entry.strip_prefix(project) {
                        actual_paths.push(relative.display().to_string());
                    }
                }
            }
        }

        let text = format!(
            "## Architecture Conformance Check\n\n\
            | Field | Value |\n\
            | ----- | ----- |\n\
            | Reference | {} |\n\
            | Files Scanned | {} |\n\
            | Patterns | {} |\n\n\
            The reference architecture document has been read. Use the document content \
            to evaluate conformance of the {} scanned files against the architecture constraints.",
            self.reference_arch_short_code,
            actual_paths.len(),
            patterns.join(", "),
            actual_paths.len()
        );
        Ok(CallToolResult {
            content: vec![TextContent::new(text, None, None).into()],
            is_error: None,
            meta: None,
            structured_content: None,
        })
    }
}
