use cadre_core::{CrossReference, InsightScope, RulesConfig, TraceabilityIndex};
use cadre_store::DocumentStore;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use std::path::Path;

pub fn tool_error(msg: impl std::fmt::Display) -> CallToolError {
    CallToolError::new(std::io::Error::other(msg.to_string()))
}

pub fn store_for(project_path: &str) -> DocumentStore {
    DocumentStore::new(Path::new(project_path))
}

pub fn build_scope_from_args(
    scope_repo: &Option<String>,
    scope_package: &Option<String>,
    scope_subsystem: &Option<String>,
    scope_paths: &Option<Vec<String>>,
    scope_symbols: &Option<Vec<String>>,
) -> InsightScope {
    let mut scope = InsightScope::new();
    scope.repo = scope_repo.clone();
    scope.package = scope_package.clone();
    scope.subsystem = scope_subsystem.clone();
    scope.paths = scope_paths.clone().unwrap_or_default();
    scope.symbols = scope_symbols.clone().unwrap_or_default();
    scope
}

pub fn build_traceability_index(
    store: &DocumentStore,
) -> Result<(TraceabilityIndex, Vec<(String, CrossReference)>), CallToolError> {
    let all_docs = store.list_documents(false).map_err(|e| tool_error(e.user_message()))?;
    let mut index = TraceabilityIndex::new();
    let mut xrefs = Vec::new();

    for doc in &all_docs {
        if doc.document_type != "cross_reference" {
            continue;
        }
        let raw = match store.read_document_raw(&doc.short_code) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(xref) = CrossReference::from_content(&raw) {
            index.add_from_document(&xref);
            xrefs.push((doc.short_code.clone(), xref));
        }
    }
    Ok((index, xrefs))
}

pub fn load_all_rules_configs(
    store: &DocumentStore,
) -> Result<Vec<RulesConfig>, CallToolError> {
    let docs = store
        .search_documents_with_options("rules_config", Some("rules_config"), None, false)
        .map_err(|e| tool_error(e.user_message()))?;

    let mut rules = Vec::new();
    for doc in &docs {
        let raw = store
            .read_document_raw(&doc.short_code)
            .map_err(|e| tool_error(e.user_message()))?;
        if let Ok(rc) = RulesConfig::from_content(&raw) {
            rules.push(rc);
        }
    }
    Ok(rules)
}

pub fn extract_tool_from_baseline(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.contains("**Tool**:") || line.contains("- **Tool**:") {
            let tool = line.split(':').nth(1)?.trim().to_string();
            return Some(tool);
        }
    }
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("title:") {
            let title = trimmed.strip_prefix("title:")?.trim().trim_matches('"');
            if title.to_lowercase().contains("clippy") {
                return Some("clippy".to_string());
            } else if title.to_lowercase().contains("eslint") {
                return Some("eslint".to_string());
            } else if title.to_lowercase().contains("tsc")
                || title.to_lowercase().contains("typescript")
            {
                return Some("tsc".to_string());
            } else if title.to_lowercase().contains("coverage") {
                return Some("coverage".to_string());
            }
        }
    }
    None
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
