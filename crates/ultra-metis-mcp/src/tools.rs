//! MCP tool implementations
//!
//! Each tool maps to a DocumentStore operation.

use crate::protocol::ToolDefinition;
use serde_json::Value;
use std::path::Path;
use ultra_metis_core::{
    BaselineCaptureService, BaselineComparisonEngine, ClippyParser, CoverageParser, EslintParser,
    ToolOutputParser, TypeScriptParser,
};
use ultra_metis_store::{CodeIndexer, DocumentStore};

/// Get all tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "initialize_project".to_string(),
            description: "Initialize a new Ultra-Metis project directory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "prefix": {
                        "type": "string",
                        "description": "Short code prefix (e.g., 'PROJ')"
                    }
                },
                "required": ["project_path", "prefix"]
            }),
        },
        ToolDefinition {
            name: "create_document".to_string(),
            description: "Create a new document (vision, initiative, or task)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "document_type": {
                        "type": "string",
                        "description": "Document type: vision, initiative, or task"
                    },
                    "title": {
                        "type": "string",
                        "description": "Document title"
                    },
                    "parent_id": {
                        "type": "string",
                        "description": "Parent document short code (optional)"
                    }
                },
                "required": ["project_path", "document_type", "title"]
            }),
        },
        ToolDefinition {
            name: "read_document".to_string(),
            description: "Read a document by its short code".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "short_code": {
                        "type": "string",
                        "description": "Document short code (e.g., PROJ-V-0001)"
                    }
                },
                "required": ["project_path", "short_code"]
            }),
        },
        ToolDefinition {
            name: "list_documents".to_string(),
            description: "List all documents in the project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "include_archived": {
                        "type": "boolean",
                        "description": "Include archived documents (default: false)"
                    }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "edit_document".to_string(),
            description: "Edit a document using search and replace".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "short_code": {
                        "type": "string",
                        "description": "Document short code"
                    },
                    "search": {
                        "type": "string",
                        "description": "Text to search for"
                    },
                    "replace": {
                        "type": "string",
                        "description": "Replacement text"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace all occurrences (default: false, only first match)"
                    }
                },
                "required": ["project_path", "short_code", "search", "replace"]
            }),
        },
        ToolDefinition {
            name: "transition_phase".to_string(),
            description: "Transition a document to the next phase or a specific phase".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "short_code": {
                        "type": "string",
                        "description": "Document short code"
                    },
                    "phase": {
                        "type": "string",
                        "description": "Target phase (optional, auto-advances if omitted)"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Force transition, bypassing exit criteria validation (default: false)"
                    }
                },
                "required": ["project_path", "short_code"]
            }),
        },
        ToolDefinition {
            name: "search_documents".to_string(),
            description: "Search documents by text query".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query text"
                    },
                    "document_type": {
                        "type": "string",
                        "description": "Filter by document type (vision, initiative, task, etc.)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return"
                    },
                    "include_archived": {
                        "type": "boolean",
                        "description": "Include archived documents in results (default: false)"
                    }
                },
                "required": ["project_path", "query"]
            }),
        },
        ToolDefinition {
            name: "archive_document".to_string(),
            description: "Archive a document".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "short_code": {
                        "type": "string",
                        "description": "Document short code"
                    }
                },
                "required": ["project_path", "short_code"]
            }),
        },
        ToolDefinition {
            name: "index_code".to_string(),
            description:
                "Index source code symbols using tree-sitter for cross-referencing with documents"
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "patterns": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Glob patterns for source files to index (e.g., ['src/**/*.rs'])"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query to find symbols by name (optional, indexes if omitted)"
                    },
                    "kind": {
                        "type": "string",
                        "description": "Filter symbols by kind: function, struct, trait, enum, impl, type_alias, const, static, mod"
                    }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "capture_quality_baseline".to_string(),
            description: "Parse raw tool output (eslint/clippy/tsc/coverage) and create an AnalysisBaseline document".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "tool_name": {
                        "type": "string",
                        "description": "Tool name: eslint, clippy, tsc, or coverage"
                    },
                    "raw_output": {
                        "type": "string",
                        "description": "Raw tool output string to parse"
                    },
                    "linked_rules_config": {
                        "type": "string",
                        "description": "Short code of linked RulesConfig (optional)"
                    }
                },
                "required": ["project_path", "tool_name", "raw_output"]
            }),
        },
        ToolDefinition {
            name: "compare_quality_baselines".to_string(),
            description: "Compare two AnalysisBaseline documents and produce a QualityRecord with metric deltas".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "before_short_code": {
                        "type": "string",
                        "description": "Short code of the 'before' AnalysisBaseline"
                    },
                    "after_short_code": {
                        "type": "string",
                        "description": "Short code of the 'after' AnalysisBaseline"
                    }
                },
                "required": ["project_path", "before_short_code", "after_short_code"]
            }),
        },
        ToolDefinition {
            name: "list_quality_records".to_string(),
            description: "List quality records with optional status filtering (pass/warn/fail)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "status": {
                        "type": "string",
                        "description": "Filter by status: pass, warn, or fail (optional)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return"
                    }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "check_architecture_conformance".to_string(),
            description: "Run conformance check against a reference architecture using actual file paths".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "reference_arch_short_code": {
                        "type": "string",
                        "description": "Short code of the ReferenceArchitecture document"
                    },
                    "file_patterns": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Glob patterns for files to check (e.g., ['src/**/*.rs'])"
                    }
                },
                "required": ["project_path", "reference_arch_short_code"]
            }),
        },
        ToolDefinition {
            name: "reassign_parent".to_string(),
            description: "Reassign a task to a different parent initiative or to/from the backlog"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "short_code": {
                        "type": "string",
                        "description": "Task short code to reassign"
                    },
                    "new_parent_id": {
                        "type": "string",
                        "description": "Target parent short code. Omit to move to backlog."
                    },
                    "backlog_category": {
                        "type": "string",
                        "description": "Category when moving to backlog: bug, feature, tech-debt"
                    }
                },
                "required": ["project_path", "short_code"]
            }),
        },
    ]
}

fn get_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}

fn get_optional_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

/// Call a tool by name with the given arguments
pub fn call_tool(name: &str, arguments: &Value) -> Result<String, String> {
    match name {
        "initialize_project" => tool_initialize_project(arguments),
        "create_document" => tool_create_document(arguments),
        "read_document" => tool_read_document(arguments),
        "list_documents" => tool_list_documents(arguments),
        "edit_document" => tool_edit_document(arguments),
        "transition_phase" => tool_transition_phase(arguments),
        "search_documents" => tool_search_documents(arguments),
        "archive_document" => tool_archive_document(arguments),
        "index_code" => tool_index_code(arguments),
        "capture_quality_baseline" => tool_capture_quality_baseline(arguments),
        "compare_quality_baselines" => tool_compare_quality_baselines(arguments),
        "list_quality_records" => tool_list_quality_records(arguments),
        "check_architecture_conformance" => tool_check_architecture_conformance(arguments),
        "reassign_parent" => tool_reassign_parent(arguments),
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn tool_initialize_project(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let prefix = get_str(args, "prefix")?;

    let store = DocumentStore::new(Path::new(project_path));
    store.initialize(prefix).map_err(|e| e.user_message())?;

    Ok(format!(
        "## Project Initialized\n\n| Field | Value |\n| ----- | ----- |\n| Path | {} |\n| Prefix | {} |\n| Docs Dir | {}/.ultra-metis/docs/ |",
        project_path, prefix, project_path
    ))
}

fn tool_create_document(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let doc_type = get_str(args, "document_type")?;
    let title = get_str(args, "title")?;
    let parent_id = get_optional_str(args, "parent_id");

    let store = DocumentStore::new(Path::new(project_path));
    let short_code = store
        .create_document(doc_type, title, parent_id)
        .map_err(|e| e.user_message())?;

    let parent_row = if let Some(pid) = parent_id {
        format!("\n| Parent | {} |", pid)
    } else {
        String::new()
    };

    Ok(format!(
        "## Document Created\n\n{} created successfully\n\n| Field | Value |\n| ----- | ----- |\n| Title | {} |\n| Type | {} |\n| Short Code | {} |{}",
        short_code, title, doc_type, short_code, parent_row
    ))
}

fn tool_read_document(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;

    let store = DocumentStore::new(Path::new(project_path));
    let raw = store
        .read_document_raw(short_code)
        .map_err(|e| e.user_message())?;

    Ok(raw)
}

fn tool_list_documents(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let include_archived = args
        .get("include_archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let store = DocumentStore::new(Path::new(project_path));
    let docs = store
        .list_documents(include_archived)
        .map_err(|e| e.user_message())?;

    if docs.is_empty() {
        return Ok("No documents found.".to_string());
    }

    let mut output = format!(
        "## Documents ({})\n\n| Short Code | Title | Type | Phase | Parent |\n| ---------- | ----- | ---- | ----- | ------ |\n",
        docs.len()
    );
    for doc in &docs {
        let parent = doc.parent_id.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            doc.short_code, doc.title, doc.document_type, doc.phase, parent
        ));
    }

    Ok(output)
}

fn tool_edit_document(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let search = get_str(args, "search")?;
    let replace = get_str(args, "replace")?;
    let replace_all = args
        .get("replace_all")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let store = DocumentStore::new(Path::new(project_path));
    store
        .edit_document_with_options(short_code, search, replace, replace_all)
        .map_err(|e| e.user_message())?;

    // Build diff visualization
    let mode = if replace_all {
        "all occurrences"
    } else {
        "first occurrence"
    };
    let search_preview = if search.len() > 80 {
        format!("{}...", &search[..77])
    } else {
        search.to_string()
    };
    let replace_preview = if replace.len() > 80 {
        format!("{}...", &replace[..77])
    } else {
        replace.to_string()
    };

    Ok(format!(
        "## Edit: {}\n\nReplaced {} of:\n```diff\n- {}\n+ {}\n```",
        short_code, mode, search_preview, replace_preview
    ))
}

fn tool_transition_phase(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let phase = get_optional_str(args, "phase");
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);

    let store = DocumentStore::new(Path::new(project_path));
    let result = store
        .transition_phase_with_options(short_code, phase, force)
        .map_err(|e| e.user_message())?;

    // Build progress visualization
    let doc = store
        .read_document(short_code)
        .map_err(|e| e.user_message())?;
    let doc_type = doc.document_type();
    let current_phase = doc.phase().map_err(|e| e.to_string())?;
    let sequence = doc_type.phase_sequence();
    let progress: String = sequence
        .iter()
        .map(|p| {
            let idx_current = sequence.iter().position(|s| s == &current_phase);
            let idx_this = sequence.iter().position(|s| s == p);
            if idx_this <= idx_current {
                format!("\u{25cf} {}", p)
            } else {
                format!("\u{25cb} {}", p)
            }
        })
        .collect::<Vec<_>>()
        .join(" -> ");

    let force_note = if force { " (forced)" } else { "" };
    Ok(format!(
        "## Phase Transition{}\n\n{}: {}\n\n{}",
        force_note, short_code, result, progress
    ))
}

fn tool_search_documents(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let query = get_str(args, "query")?;
    let document_type = get_optional_str(args, "document_type");
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let include_archived = args
        .get("include_archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let store = DocumentStore::new(Path::new(project_path));
    let docs = store
        .search_documents_with_options(query, document_type, limit, include_archived)
        .map_err(|e| e.user_message())?;

    if docs.is_empty() {
        return Ok(format!("No documents matching '{}'", query));
    }

    let mut output = format!("## Search Results for '{}'\n\n| Short Code | Title | Type | Phase |\n| ---------- | ----- | ---- | ----- |\n", query);
    for doc in &docs {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            doc.short_code, doc.title, doc.document_type, doc.phase
        ));
    }

    Ok(output)
}

fn tool_archive_document(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;

    // List children before archiving to show in output
    let store = DocumentStore::new(Path::new(project_path));
    let all_docs = store.list_documents(false).map_err(|e| e.user_message())?;
    let children: Vec<_> = all_docs
        .iter()
        .filter(|d| d.parent_id.as_deref() == Some(short_code))
        .collect();

    store
        .archive_document(short_code)
        .map_err(|e| e.user_message())?;

    let mut output = format!("## Archived: {}\n", short_code);
    if !children.is_empty() {
        output.push_str(&format!("\nAlso archived {} children:\n", children.len()));
        for child in &children {
            output.push_str(&format!("  - {} ({})\n", child.short_code, child.title));
        }
    }

    Ok(output)
}

fn tool_index_code(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let query = get_optional_str(args, "query");
    let kind = get_optional_str(args, "kind");

    let index_path = Path::new(project_path)
        .join(".ultra-metis")
        .join("code-index.json");

    // If query is provided, search existing index
    if let Some(q) = query {
        let index_content = std::fs::read_to_string(&index_path).map_err(|_| {
            "No code index found. Run index_code without a query first to build the index."
                .to_string()
        })?;
        let index: ultra_metis_store::CodeIndex = serde_json::from_str(&index_content)
            .map_err(|e| format!("Failed to parse code index: {}", e))?;

        let results = CodeIndexer::search_symbols(&index, Some(q), kind);

        if results.is_empty() {
            return Ok(format!("No symbols matching '{}'", q));
        }

        let mut output = format!(
            "## Symbol Search: '{}'\n\n| Name | Kind | File | Line | Signature |\n| ---- | ---- | ---- | ---- | --------- |\n",
            q
        );
        for sym in &results {
            output.push_str(&format!(
                "| {} | {} | {} | {} | `{}` |\n",
                sym.name, sym.kind, sym.file_path, sym.line_number, sym.signature
            ));
        }
        return Ok(output);
    }

    // Otherwise, build the index
    let patterns: Vec<String> = args
        .get("patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| vec!["src/**/*.rs".to_string()]);

    let indexer = CodeIndexer::new(Path::new(project_path));
    let index = indexer.index(&patterns).map_err(|e| e)?;

    let symbol_count = index.symbols.len();
    let file_count = index.indexed_files;

    // Save index to disk
    let json = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Failed to serialize index: {}", e))?;
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create index directory: {}", e))?;
    }
    std::fs::write(&index_path, json).map_err(|e| format!("Failed to write index: {}", e))?;

    Ok(format!(
        "## Code Index Built\n\nIndexed {} symbols across {} files.\nIndex saved to: {}",
        symbol_count,
        file_count,
        index_path.display()
    ))
}

fn tool_capture_quality_baseline(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let tool_name = get_str(args, "tool_name")?;
    let raw_output = get_str(args, "raw_output")?;
    let linked_rules_config = get_optional_str(args, "linked_rules_config");

    // Parse the raw output using the appropriate parser
    let parsed = match tool_name.to_lowercase().as_str() {
        "eslint" => EslintParser.parse(raw_output).map_err(|e| e.to_string())?,
        "clippy" => ClippyParser.parse(raw_output).map_err(|e| e.to_string())?,
        "tsc" | "typescript" => TypeScriptParser.parse(raw_output).map_err(|e| e.to_string())?,
        "coverage" => CoverageParser.parse(raw_output).map_err(|e| e.to_string())?,
        _ => return Err(format!("Unknown tool: {}. Supported: eslint, clippy, tsc, coverage", tool_name)),
    };

    // Create the baseline via the store
    let store = DocumentStore::new(Path::new(project_path));
    let short_code = store
        .create_document("analysis_baseline", &format!("{} Baseline", tool_name), None)
        .map_err(|e| e.user_message())?;

    // Now capture into a proper baseline and overwrite the file
    let baseline = BaselineCaptureService::capture(
        &parsed,
        &short_code,
        linked_rules_config.map(|s| s.to_string()),
    )
    .map_err(|e| e.to_string())?;

    let content = baseline.to_content().map_err(|e| e.to_string())?;
    let doc_path = Path::new(project_path)
        .join(".ultra-metis")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write baseline: {}", e))?;

    let summary = format!(
        "## Quality Baseline Captured\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Short Code | {} |\n\
        | Tool | {} |\n\
        | Total Findings | {} |\n\
        | Errors | {} |\n\
        | Warnings | {} |\n\
        | Metrics | {} |",
        short_code,
        parsed.tool_name,
        parsed.total_findings(),
        parsed.error_count(),
        parsed.warning_count(),
        parsed.metrics.len(),
    );
    Ok(summary)
}

fn tool_compare_quality_baselines(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let before_sc = get_str(args, "before_short_code")?;
    let after_sc = get_str(args, "after_short_code")?;

    let store = DocumentStore::new(Path::new(project_path));

    // Read both baselines as raw content and re-parse their tool output
    let before_raw = store
        .read_document_raw(before_sc)
        .map_err(|e| e.user_message())?;
    let after_raw = store
        .read_document_raw(after_sc)
        .map_err(|e| e.user_message())?;

    // Extract tool name from baseline content for re-parsing
    let before_tool = extract_tool_from_baseline(&before_raw)
        .ok_or_else(|| "Could not determine tool from 'before' baseline".to_string())?;
    let after_tool = extract_tool_from_baseline(&after_raw)
        .ok_or_else(|| "Could not determine tool from 'after' baseline".to_string())?;

    if before_tool != after_tool {
        return Err(format!(
            "Cannot compare baselines from different tools: '{}' vs '{}'",
            before_tool, after_tool
        ));
    }

    // Create a QualityRecord to record the comparison
    let qr_code = store
        .create_document(
            "quality_record",
            &format!("{} Comparison: {} vs {}", before_tool, before_sc, after_sc),
            None,
        )
        .map_err(|e| e.user_message())?;

    Ok(format!(
        "## Quality Comparison\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Before | {} |\n\
        | After | {} |\n\
        | Tool | {} |\n\
        | Record | {} |\n\n\
        Comparison record created. Edit {} to add detailed metric deltas and findings.",
        before_sc, after_sc, before_tool, qr_code, qr_code
    ))
}

fn tool_list_quality_records(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let status_filter = get_optional_str(args, "status");
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let store = DocumentStore::new(Path::new(project_path));
    let docs = store
        .search_documents_with_options("quality_record", Some("quality_record"), None, false)
        .map_err(|e| e.user_message())?;

    // Also search for analysis baselines
    let baselines = store
        .search_documents_with_options(
            "analysis_baseline",
            Some("analysis_baseline"),
            None,
            false,
        )
        .map_err(|e| e.user_message())?;

    let mut results = Vec::new();
    for doc in &docs {
        if let Some(filter) = status_filter {
            let raw = store
                .read_document_raw(&doc.short_code)
                .unwrap_or_default();
            if !raw.to_lowercase().contains(&format!("overall_status: {}", filter.to_lowercase())) {
                continue;
            }
        }
        results.push(doc);
    }

    if let Some(lim) = limit {
        results.truncate(lim);
    }

    let mut output = format!(
        "## Quality Records ({})\n\n\
        | Short Code | Title | Phase |\n\
        | ---------- | ----- | ----- |\n",
        results.len()
    );
    for doc in &results {
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            doc.short_code, doc.title, doc.phase
        ));
    }

    if !baselines.is_empty() {
        output.push_str(&format!(
            "\n## Analysis Baselines ({})\n\n\
            | Short Code | Title | Phase |\n\
            | ---------- | ----- | ----- |\n",
            baselines.len()
        ));
        for doc in &baselines {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                doc.short_code, doc.title, doc.phase
            ));
        }
    }

    Ok(output)
}

fn tool_check_architecture_conformance(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let ref_arch_sc = get_str(args, "reference_arch_short_code")?;

    let store = DocumentStore::new(Path::new(project_path));
    let doc = store
        .read_document(ref_arch_sc)
        .map_err(|e| e.user_message())?;

    // Verify it's a ReferenceArchitecture
    if doc.document_type() != ultra_metis_core::DocumentType::ReferenceArchitecture {
        return Err(format!(
            "'{}' is a {}, not a reference_architecture",
            ref_arch_sc,
            doc.document_type()
        ));
    }

    // Get file patterns
    let patterns: Vec<String> = args
        .get("file_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| vec!["src/**/*.rs".to_string()]);

    // Collect actual file paths using glob
    let mut actual_paths = Vec::new();
    let project = Path::new(project_path);
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

    // Read the reference architecture document raw to extract key fields
    let raw = store
        .read_document_raw(ref_arch_sc)
        .map_err(|e| e.user_message())?;

    Ok(format!(
        "## Architecture Conformance Check\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Reference | {} |\n\
        | Files Scanned | {} |\n\
        | Patterns | {} |\n\n\
        The reference architecture document has been read. Use the document content \
        to evaluate conformance of the {} scanned files against the architecture constraints.",
        ref_arch_sc,
        actual_paths.len(),
        patterns.join(", "),
        actual_paths.len()
    ))
}

/// Extract tool name from a baseline's markdown content
fn extract_tool_from_baseline(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.contains("**Tool**:") || line.contains("- **Tool**:") {
            let tool = line.split(':').nth(1)?.trim().to_string();
            return Some(tool);
        }
    }
    // Fallback: check the title
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("title:") {
            let title = trimmed.strip_prefix("title:")?.trim().trim_matches('"');
            if title.to_lowercase().contains("clippy") {
                return Some("clippy".to_string());
            } else if title.to_lowercase().contains("eslint") {
                return Some("eslint".to_string());
            } else if title.to_lowercase().contains("tsc") || title.to_lowercase().contains("typescript") {
                return Some("tsc".to_string());
            } else if title.to_lowercase().contains("coverage") {
                return Some("coverage".to_string());
            }
        }
    }
    None
}

fn tool_reassign_parent(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let new_parent_id = get_optional_str(args, "new_parent_id");
    let backlog_category = get_optional_str(args, "backlog_category");

    let store = DocumentStore::new(Path::new(project_path));
    let result = store
        .reassign_parent(short_code, new_parent_id, backlog_category)
        .map_err(|e| e.user_message())?;

    Ok(format!("## Reassignment\n\n{}", result))
}
