use cadre_core::{BootstrapResult, CrossReference, InsightScope, RulesConfig, TraceabilityIndex};
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

pub fn format_bootstrap_result(result: &BootstrapResult) -> String {
    let mut out = String::new();

    // Header table
    let classification = if result.is_brownfield {
        "Brownfield"
    } else {
        "Greenfield"
    };
    out.push_str("## Project Analysis\n\n");
    out.push_str("| Field | Value |\n");
    out.push_str("| ----- | ----- |\n");
    out.push_str(&format!("| Classification | {classification} |\n"));
    out.push_str(&format!("| Project Type | {} |\n", result.project_type));

    // Detected Languages
    format_languages_section(&mut out, result);

    // Build Tools
    format_build_tools_section(&mut out, result);

    // Dev Tools
    format_dev_tools_section(&mut out, result);

    // Monorepo
    format_monorepo_section(&mut out, result);

    // Summary
    out.push_str(&format!(
        "\n### Summary\n{}\n",
        result.summary.description
    ));

    // Suggestions
    if !result.summary.suggestions.is_empty() {
        out.push_str("\n### Suggestions\n");
        for suggestion in &result.summary.suggestions {
            out.push_str(&format!("- {suggestion}\n"));
        }
    }

    out
}

fn format_languages_section(out: &mut String, result: &BootstrapResult) {
    if !result.scan.languages.is_empty() {
        out.push_str("\n### Detected Languages\n");
        out.push_str("| Language | Files |\n");
        out.push_str("| -------- | ----- |\n");
        for lang in &result.scan.languages {
            // Sum file counts from extension_counts for this language's extensions
            let file_count = count_files_for_language(
                &lang.name,
                &result.scan.extension_counts,
            );
            out.push_str(&format!(
                "| {} | {} |\n",
                capitalize_first(&lang.name),
                file_count,
            ));
        }
    }
}

fn count_files_for_language(
    language: &str,
    extension_counts: &[(String, usize)],
) -> usize {
    let extensions: &[&str] = match language {
        "rust" => &["rs"],
        "javascript" => &["js", "jsx", "ts", "tsx", "mjs", "cjs"],
        "typescript" => &["ts", "tsx"],
        "python" => &["py"],
        "go" => &["go"],
        "java" => &["java"],
        "kotlin" => &["kt", "kts"],
        "ruby" => &["rb"],
        "php" => &["php"],
        "csharp" => &["cs"],
        "fsharp" => &["fs"],
        "scala" => &["scala"],
        "swift" => &["swift"],
        "c" => &["c", "h"],
        "cpp" => &["cpp", "hpp", "cc", "cxx"],
        _ => &[],
    };
    extension_counts
        .iter()
        .filter(|(ext, _)| extensions.contains(&ext.as_str()))
        .map(|(_, count)| count)
        .sum()
}

fn format_build_tools_section(out: &mut String, result: &BootstrapResult) {
    if !result.scan.build_tools.is_empty() {
        out.push_str("\n### Build Tools\n");
        for tool in &result.scan.build_tools {
            out.push_str(&format!("- {}\n", capitalize_first(&tool.to_string())));
        }
    }
}

fn format_dev_tools_section(out: &mut String, result: &BootstrapResult) {
    if !result.tools.tools.is_empty() {
        out.push_str("\n### Dev Tools\n");
        out.push_str("| Tool | Category |\n");
        out.push_str("| ---- | -------- |\n");
        for tool in &result.tools.tools {
            out.push_str(&format!(
                "| {} | {} |\n",
                tool.name,
                capitalize_first(&tool.category.to_string()),
            ));
        }
    }
}

fn format_monorepo_section(out: &mut String, result: &BootstrapResult) {
    out.push_str("\n### Monorepo\n");
    if result.monorepo.is_monorepo {
        let tool_names: Vec<String> =
            result.monorepo.tools.iter().map(std::string::ToString::to_string).collect();
        out.push_str(&format!(
            "Yes ({})\n",
            if tool_names.is_empty() {
                "structural".to_string()
            } else {
                tool_names.join(", ")
            },
        ));
        if !result.monorepo.packages.is_empty() {
            out.push_str("\n| Package | Kind | Path |\n");
            out.push_str("| ------- | ---- | ---- |\n");
            for pkg in &result.monorepo.packages {
                out.push_str(&format!(
                    "| {} | {} | {} |\n",
                    pkg.name,
                    capitalize_first(&pkg.kind.to_string()),
                    pkg.path,
                ));
            }
        }
    } else {
        out.push_str("No\n");
    }
}
