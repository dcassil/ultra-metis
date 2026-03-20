//! MCP tool implementations
//!
//! Each tool maps to a DocumentStore operation.

use crate::protocol::ToolDefinition;
use serde_json::Value;
use std::path::Path;
use ultra_metis_core::{
    BaselineCaptureService, BaselineComparisonEngine, BrownfieldEvaluator, CatalogQueryEngine,
    ClippyParser, CoverageParser, CrossReference, DurableInsightNote, EslintParser,
    EvaluatorConfig, FeedbackSignal, InsightCategory, InsightScope, RelationshipType, RulesConfig,
    ToolOutputParser, TraceabilityIndex, TypeScriptParser,
};
use ultra_metis_core::domain::rules::query::{RuleQuery, RuleQueryEngine, scope_hierarchy};
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
            name: "query_rules".to_string(),
            description: "Query engineering rules by scope, category, and protection level".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "scope": {
                        "type": "string",
                        "description": "Filter by scope: platform, org, repo, package, component, task (optional)"
                    },
                    "protection_level": {
                        "type": "string",
                        "description": "Filter by protection level: standard or protected (optional)"
                    },
                    "source_architecture_ref": {
                        "type": "string",
                        "description": "Filter by source architecture short code (optional)"
                    },
                    "include_archived": {
                        "type": "boolean",
                        "description": "Include archived rules (default: false)"
                    }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "get_applicable_rules".to_string(),
            description: "Get all rules that apply at a given scope via inheritance (broadest to narrowest)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    },
                    "target_scope": {
                        "type": "string",
                        "description": "Target scope level: platform, org, repo, package, component, task"
                    }
                },
                "required": ["project_path", "target_scope"]
            }),
        },
        ToolDefinition {
            name: "create_insight_note".to_string(),
            description: "Create a durable insight note capturing reusable local knowledge".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "title": { "type": "string", "description": "Short descriptive title for the insight" },
                    "note": { "type": "string", "description": "The insight text — what the agent should know" },
                    "category": { "type": "string", "description": "Category: hotspot_warning, recurring_failure, misleading_name, validation_hint, local_exception, boundary_warning, subsystem_quirk" },
                    "scope_repo": { "type": "string", "description": "Repository name (optional)" },
                    "scope_package": { "type": "string", "description": "Package/crate name (optional)" },
                    "scope_subsystem": { "type": "string", "description": "Logical subsystem label (optional)" },
                    "scope_paths": { "type": "array", "items": { "type": "string" }, "description": "File paths the insight applies to (optional)" },
                    "scope_symbols": { "type": "array", "items": { "type": "string" }, "description": "Symbol names the insight applies to (optional)" }
                },
                "required": ["project_path", "title", "note", "category"]
            }),
        },
        ToolDefinition {
            name: "fetch_insight_notes".to_string(),
            description: "Fetch insight notes relevant to a given scope (call at task start to load contextual knowledge)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "scope_repo": { "type": "string", "description": "Repository name (optional)" },
                    "scope_package": { "type": "string", "description": "Package/crate name (optional)" },
                    "scope_subsystem": { "type": "string", "description": "Logical subsystem label (optional)" },
                    "scope_paths": { "type": "array", "items": { "type": "string" }, "description": "File paths to match (optional)" },
                    "scope_symbols": { "type": "array", "items": { "type": "string" }, "description": "Symbol names to match (optional)" },
                    "limit": { "type": "integer", "description": "Max notes to return (default: 10)" }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "score_insight_note".to_string(),
            description: "Record feedback on an insight note after using it (helpful/meh/harmful)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "short_code": { "type": "string", "description": "Short code of the insight note" },
                    "signal": { "type": "string", "description": "Feedback signal: helpful, meh, or harmful" }
                },
                "required": ["project_path", "short_code", "signal"]
            }),
        },
        ToolDefinition {
            name: "list_insight_notes".to_string(),
            description: "List insight notes with optional status and category filtering".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "status": { "type": "string", "description": "Filter by status: active, prune_candidate, needs_human_review, archived (optional)" },
                    "category": { "type": "string", "description": "Filter by category (optional)" },
                    "include_archived": { "type": "boolean", "description": "Include archived notes (default: false)" }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "create_cross_reference".to_string(),
            description: "Create a typed relationship between two documents".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "source_ref": { "type": "string", "description": "Short code of the source document" },
                    "target_ref": { "type": "string", "description": "Short code of the target document" },
                    "relationship_type": { "type": "string", "description": "Type: parent_child, governs, references, derived_from, supersedes, conflicts_with, validates, blocks, approved_by" },
                    "description": { "type": "string", "description": "Human-readable description of the relationship (optional)" },
                    "bidirectional": { "type": "boolean", "description": "Whether traversable in both directions (default: false)" }
                },
                "required": ["project_path", "source_ref", "target_ref", "relationship_type"]
            }),
        },
        ToolDefinition {
            name: "query_relationships".to_string(),
            description: "Query all relationships involving a specific document".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "short_code": { "type": "string", "description": "Document short code to query relationships for" },
                    "direction": { "type": "string", "description": "Direction: outgoing, incoming, or all (default: all)" },
                    "relationship_type": { "type": "string", "description": "Filter by relationship type (optional)" }
                },
                "required": ["project_path", "short_code"]
            }),
        },
        ToolDefinition {
            name: "trace_ancestry".to_string(),
            description: "Walk the document hierarchy to find ancestors, descendants, or siblings".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "short_code": { "type": "string", "description": "Document short code to trace from" },
                    "direction": { "type": "string", "description": "Direction: ancestors, descendants, or siblings" }
                },
                "required": ["project_path", "short_code", "direction"]
            }),
        },
        ToolDefinition {
            name: "query_architecture_catalog".to_string(),
            description: "Search the architecture catalog by language and project type".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "language": { "type": "string", "description": "Filter by language (optional)" },
                    "project_type": { "type": "string", "description": "Filter by project type (optional)" }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "list_catalog_languages".to_string(),
            description: "List all available languages and project types in the architecture catalog".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "read_reference_architecture".to_string(),
            description: "Read the project's selected reference architecture document".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "short_code": { "type": "string", "description": "Short code of the ReferenceArchitecture (optional, finds active one if omitted)" }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "evaluate_brownfield".to_string(),
            description: "Evaluate how well the current repo matches a catalog architecture entry".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "language": { "type": "string", "description": "Language to match against" },
                    "project_type": { "type": "string", "description": "Project type to match against" }
                },
                "required": ["project_path", "language", "project_type"]
            }),
        },
        ToolDefinition {
            name: "list_cross_references".to_string(),
            description: "List all cross-references with optional filtering".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Path to the project root directory" },
                    "relationship_type": { "type": "string", "description": "Filter by relationship type (optional)" },
                    "involving": { "type": "string", "description": "Filter to show only references involving this short code (optional)" }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "list_protected_rules".to_string(),
            description: "List all protected rules for governance auditing".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project root directory"
                    }
                },
                "required": ["project_path"]
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
        "query_rules" => tool_query_rules(arguments),
        "get_applicable_rules" => tool_get_applicable_rules(arguments),
        "create_insight_note" => tool_create_insight_note(arguments),
        "fetch_insight_notes" => tool_fetch_insight_notes(arguments),
        "score_insight_note" => tool_score_insight_note(arguments),
        "list_insight_notes" => tool_list_insight_notes(arguments),
        "create_cross_reference" => tool_create_cross_reference(arguments),
        "query_relationships" => tool_query_relationships(arguments),
        "trace_ancestry" => tool_trace_ancestry(arguments),
        "list_cross_references" => tool_list_cross_references(arguments),
        "query_architecture_catalog" => tool_query_architecture_catalog(arguments),
        "list_catalog_languages" => tool_list_catalog_languages(arguments),
        "read_reference_architecture" => tool_read_reference_architecture(arguments),
        "evaluate_brownfield" => tool_evaluate_brownfield(arguments),
        "list_protected_rules" => tool_list_protected_rules(arguments),
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

// ---------------------------------------------------------------------------
// Insight Note tool handlers
// ---------------------------------------------------------------------------

fn build_scope_from_args(args: &Value) -> InsightScope {
    let mut scope = InsightScope::new();
    scope.repo = get_optional_str(args, "scope_repo").map(|s| s.to_string());
    scope.package = get_optional_str(args, "scope_package").map(|s| s.to_string());
    scope.subsystem = get_optional_str(args, "scope_subsystem").map(|s| s.to_string());
    scope.paths = args
        .get("scope_paths")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    scope.symbols = args
        .get("scope_symbols")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    scope
}

fn tool_create_insight_note(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let title = get_str(args, "title")?;
    let note_text = get_str(args, "note")?;
    let category_str = get_str(args, "category")?;

    let category: InsightCategory = category_str
        .parse()
        .map_err(|e: String| e)?;

    let scope = build_scope_from_args(args);

    let store = DocumentStore::new(Path::new(project_path));
    let short_code = store
        .create_document("durable_insight_note", title, None)
        .map_err(|e| e.user_message())?;

    // Now create the proper note with scope and category, overwrite
    let din = DurableInsightNote::new(
        title.to_string(),
        note_text.to_string(),
        category,
        scope.clone(),
        vec![ultra_metis_core::Tag::Phase(ultra_metis_core::Phase::Draft)],
        false,
        short_code.clone(),
    )
    .map_err(|e| e.to_string())?;

    let content = din.to_content().map_err(|e| e.to_string())?;
    let doc_path = Path::new(project_path)
        .join(".ultra-metis")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write note: {}", e))?;

    let scope_desc = [
        scope.repo.as_deref().unwrap_or(""),
        scope.package.as_deref().unwrap_or(""),
        scope.subsystem.as_deref().unwrap_or(""),
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .cloned()
    .collect::<Vec<_>>()
    .join(", ");

    Ok(format!(
        "## Insight Note Created\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Short Code | {} |\n\
        | Title | {} |\n\
        | Category | {} |\n\
        | Scope | {} |",
        short_code,
        title,
        category_str,
        if scope_desc.is_empty() { "global" } else { &scope_desc }
    ))
}

fn tool_fetch_insight_notes(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let query_scope = build_scope_from_args(args);
    let store = DocumentStore::new(Path::new(project_path));

    // Load all DurableInsightNote documents
    let all_docs = store
        .list_documents(false)
        .map_err(|e| e.user_message())?;

    let mut matched = Vec::new();
    for doc in &all_docs {
        if doc.document_type != "durable_insight_note" {
            continue;
        }
        let raw = match store.read_document_raw(&doc.short_code) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(din) = DurableInsightNote::from_content(&raw) {
            if din.status != ultra_metis_core::NoteStatus::Active {
                continue;
            }
            // Check scope match (if query scope has any fields set)
            let has_query = query_scope.repo.is_some()
                || query_scope.package.is_some()
                || query_scope.subsystem.is_some()
                || !query_scope.paths.is_empty()
                || !query_scope.symbols.is_empty();

            if !has_query || din.scope.matches(&query_scope) {
                matched.push((doc.short_code.clone(), din));
            }
        }
        if matched.len() >= limit {
            break;
        }
    }

    // Record fetch on each matched note and save back
    for (sc, din) in &mut matched {
        let mut note = DurableInsightNote::from_content(
            &store.read_document_raw(sc).unwrap_or_default(),
        ).ok();
        if let Some(ref mut n) = note {
            n.record_fetch();
            if let Ok(content) = n.to_content() {
                let doc_path = Path::new(project_path)
                    .join(".ultra-metis")
                    .join("docs")
                    .join(format!("{}.md", sc));
                let _ = std::fs::write(&doc_path, content);
            }
        }
    }

    if matched.is_empty() {
        return Ok("No insight notes found for the given scope.".to_string());
    }

    let mut output = format!(
        "## Insight Notes ({})\n\n",
        matched.len()
    );
    for (sc, din) in &matched {
        output.push_str(&format!(
            "### {} — {} [{}]\n\n{}\n\n---\n\n",
            sc, din.title(), din.category, din.note
        ));
    }
    Ok(output)
}

fn tool_score_insight_note(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let signal_str = get_str(args, "signal")?;

    let signal: FeedbackSignal = signal_str.parse().map_err(|e: String| e)?;

    let store = DocumentStore::new(Path::new(project_path));
    let raw = store.read_document_raw(short_code).map_err(|e| e.user_message())?;
    let mut din = DurableInsightNote::from_content(&raw).map_err(|e| e.to_string())?;

    din.record_feedback(signal);

    // Check for prune candidate
    let was_pruned = din.should_be_prune_candidate(30, 0.5, 3, 5, 0.3);
    if was_pruned {
        din.mark_prune_candidate();
    }

    let content = din.to_content().map_err(|e| e.to_string())?;
    let doc_path = Path::new(project_path)
        .join(".ultra-metis")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write: {}", e))?;

    let status_change = if was_pruned {
        " (marked as prune candidate)"
    } else {
        ""
    };

    Ok(format!(
        "## Feedback Recorded{}\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Note | {} |\n\
        | Signal | {} |\n\
        | Total Helpful | {} |\n\
        | Total Meh | {} |\n\
        | Total Harmful | {} |\n\
        | Status | {} |",
        status_change,
        short_code,
        signal_str,
        din.thumbs_up_count,
        din.meh_count,
        din.thumbs_down_count,
        din.status
    ))
}

fn tool_list_insight_notes(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let status_filter = get_optional_str(args, "status");
    let category_filter = get_optional_str(args, "category");
    let include_archived = args
        .get("include_archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let store = DocumentStore::new(Path::new(project_path));
    let all_docs = store
        .list_documents(include_archived)
        .map_err(|e| e.user_message())?;

    let mut notes = Vec::new();
    for doc in &all_docs {
        if doc.document_type != "durable_insight_note" {
            continue;
        }
        let raw = match store.read_document_raw(&doc.short_code) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(din) = DurableInsightNote::from_content(&raw) {
            // Status filter
            if let Some(sf) = status_filter {
                if din.status.to_string() != sf {
                    continue;
                }
            }
            // Category filter
            if let Some(cf) = category_filter {
                if din.category.to_string() != cf {
                    continue;
                }
            }
            notes.push((doc.short_code.clone(), din));
        }
    }

    if notes.is_empty() {
        return Ok("No insight notes found.".to_string());
    }

    let mut output = format!(
        "## Insight Notes ({})\n\n\
        | Short Code | Title | Category | Status | Fetches | Helpful% |\n\
        | ---------- | ----- | -------- | ------ | ------- | -------- |\n",
        notes.len()
    );
    for (sc, din) in &notes {
        let helpful_pct = if din.total_feedback() > 0 {
            format!("{:.0}%", din.helpful_ratio() * 100.0)
        } else {
            "-".to_string()
        };
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            sc, din.title(), din.category, din.status, din.fetch_count, helpful_pct
        ));
    }
    Ok(output)
}

// ---------------------------------------------------------------------------
// Traceability tool handlers
// ---------------------------------------------------------------------------

/// Helper to build a TraceabilityIndex from all CrossReference documents in the store
fn build_traceability_index(store: &DocumentStore) -> Result<(TraceabilityIndex, Vec<(String, CrossReference)>), String> {
    let all_docs = store.list_documents(false).map_err(|e| e.user_message())?;
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

fn tool_create_cross_reference(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let source_ref = get_str(args, "source_ref")?;
    let target_ref = get_str(args, "target_ref")?;
    let rel_type_str = get_str(args, "relationship_type")?;
    let description = get_optional_str(args, "description").unwrap_or("");
    let bidirectional = args.get("bidirectional").and_then(|v| v.as_bool()).unwrap_or(false);

    if source_ref == target_ref {
        return Err("Source and target cannot be the same document.".to_string());
    }

    let rel_type: RelationshipType = rel_type_str
        .parse()
        .map_err(|e: String| e)?;

    let store = DocumentStore::new(Path::new(project_path));

    // Verify both documents exist
    store.read_document(source_ref).map_err(|e| e.user_message())?;
    store.read_document(target_ref).map_err(|e| e.user_message())?;

    let title = format!("{} {} {}", source_ref, rel_type, target_ref);
    let short_code = store
        .create_document("cross_reference", &title, None)
        .map_err(|e| e.user_message())?;

    // Create the proper cross-reference and overwrite
    let xref = CrossReference::new(
        title.clone(),
        vec![ultra_metis_core::Tag::Phase(ultra_metis_core::Phase::Draft)],
        false,
        short_code.clone(),
        source_ref.to_string(),
        target_ref.to_string(),
        rel_type,
        description.to_string(),
        bidirectional,
    )
    .map_err(|e| e.to_string())?;

    let content = xref.to_content().map_err(|e| e.to_string())?;
    let doc_path = Path::new(project_path)
        .join(".ultra-metis")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write: {}", e))?;

    let bidir_label = if bidirectional { " (bidirectional)" } else { "" };
    Ok(format!(
        "## Cross-Reference Created{}\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Short Code | {} |\n\
        | Source | {} |\n\
        | Target | {} |\n\
        | Type | {} |\n\
        | Description | {} |",
        bidir_label, short_code, source_ref, target_ref, rel_type, description
    ))
}

fn tool_query_relationships(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let direction = get_optional_str(args, "direction").unwrap_or("all");
    let rel_type_filter = get_optional_str(args, "relationship_type");

    let store = DocumentStore::new(Path::new(project_path));
    let (index, xrefs) = build_traceability_index(&store)?;

    let entries = match direction {
        "outgoing" => index.outgoing(short_code),
        "incoming" => index.incoming(short_code),
        _ => index.involving(short_code),
    };

    let filtered: Vec<_> = if let Some(rt) = rel_type_filter {
        let rel_type: RelationshipType = rt.parse().map_err(|e: String| e)?;
        entries.into_iter().filter(|e| e.relationship_type == rel_type).collect()
    } else {
        entries
    };

    if filtered.is_empty() {
        return Ok(format!("No {} relationships found for {}.", direction, short_code));
    }

    let mut output = format!(
        "## Relationships for {} ({}, {})\n\n\
        | Source | Type | Target | Bidirectional |\n\
        | ------ | ---- | ------ | ------------- |\n",
        short_code, direction, filtered.len()
    );
    for entry in &filtered {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            entry.source_ref, entry.relationship_type, entry.target_ref, entry.bidirectional
        ));
    }
    Ok(output)
}

fn tool_trace_ancestry(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let direction = get_str(args, "direction")?;

    let store = DocumentStore::new(Path::new(project_path));
    let (index, _) = build_traceability_index(&store)?;

    let results = match direction {
        "ancestors" => index.ancestors(short_code),
        "descendants" => index.descendants(short_code),
        "siblings" => index.siblings(short_code),
        _ => return Err(format!("Invalid direction '{}'. Use: ancestors, descendants, or siblings", direction)),
    };

    if results.is_empty() {
        return Ok(format!("No {} found for {}.", direction, short_code));
    }

    let mut output = format!(
        "## {} of {} ({})\n\n",
        capitalize_first(direction), short_code, results.len()
    );
    for (i, sc) in results.iter().enumerate() {
        let title = store
            .read_document(sc)
            .map(|d| d.title().to_string())
            .unwrap_or_else(|_| "?".to_string());
        let prefix = if direction == "ancestors" {
            "  ".repeat(i)
        } else {
            "  ".repeat(0)
        };
        output.push_str(&format!("{}- {} ({})\n", prefix, sc, title));
    }
    Ok(output)
}

fn tool_list_cross_references(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let rel_type_filter = get_optional_str(args, "relationship_type");
    let involving = get_optional_str(args, "involving");

    let store = DocumentStore::new(Path::new(project_path));
    let (_, xrefs) = build_traceability_index(&store)?;

    let filtered: Vec<_> = xrefs
        .iter()
        .filter(|(_, xref)| {
            if let Some(rt) = rel_type_filter {
                if let Ok(rel_type) = rt.parse::<RelationshipType>() {
                    if xref.relationship_type != rel_type {
                        return false;
                    }
                }
            }
            if let Some(inv) = involving {
                if !xref.involves(inv) {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        return Ok("No cross-references found.".to_string());
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
    Ok(output)
}

// ---------------------------------------------------------------------------
// Architecture Catalog tool handlers
// ---------------------------------------------------------------------------

fn tool_query_architecture_catalog(args: &Value) -> Result<String, String> {
    let _project_path = get_str(args, "project_path")?;
    let language = get_optional_str(args, "language");
    let project_type = get_optional_str(args, "project_type");

    let engine = CatalogQueryEngine::with_builtins();

    let mut query = ultra_metis_core::CatalogQuery::new();
    // For catalog queries, match all phases since builtins are Published
    if let Some(lang) = language {
        query = query.with_language(lang);
    }
    if let Some(pt) = project_type {
        query = query.with_project_type(pt);
    }

    let matches = engine.query(&query);

    if matches.is_empty() {
        let filter_desc = match (language, project_type) {
            (Some(l), Some(p)) => format!("language='{}', project_type='{}'", l, p),
            (Some(l), None) => format!("language='{}'", l),
            (None, Some(p)) => format!("project_type='{}'", p),
            (None, None) => "no filters".to_string(),
        };
        return Ok(format!("No catalog entries found for {}.", filter_desc));
    }

    let mut output = format!(
        "## Architecture Catalog ({} entries)\n\n",
        matches.len()
    );
    for m in &matches {
        let entry = m.entry;
        output.push_str(&format!(
            "### {} — {} / {}\n\n",
            entry.title(), entry.language, entry.project_type
        ));
        if !entry.folder_layout.is_empty() {
            output.push_str(&format!("**Folder Layout**: {}\n\n", entry.folder_layout.join(", ")));
        }
        if !entry.layers.is_empty() {
            output.push_str(&format!("**Layers**: {}\n\n", entry.layers.join(", ")));
        }
        if !entry.dependency_rules.is_empty() {
            output.push_str(&format!("**Dependency Rules**: {}\n\n", entry.dependency_rules.join("; ")));
        }
        if !entry.naming_conventions.is_empty() {
            output.push_str(&format!("**Naming Conventions**: {}\n\n", entry.naming_conventions.join("; ")));
        }
        output.push_str("---\n\n");
    }
    Ok(output)
}

fn tool_list_catalog_languages(args: &Value) -> Result<String, String> {
    let _project_path = get_str(args, "project_path")?;

    let engine = CatalogQueryEngine::with_builtins();
    let languages = engine.languages();

    if languages.is_empty() {
        return Ok("No languages found in the catalog.".to_string());
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
    Ok(output)
}

fn tool_read_reference_architecture(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_optional_str(args, "short_code");

    let store = DocumentStore::new(Path::new(project_path));

    let sc = if let Some(sc) = short_code {
        sc.to_string()
    } else {
        // Find the first (active) reference architecture
        let all_docs = store.list_documents(false).map_err(|e| e.user_message())?;
        let ra = all_docs
            .iter()
            .find(|d| d.document_type == "reference_architecture");
        match ra {
            Some(d) => d.short_code.clone(),
            None => return Ok("No reference architecture found. Create one first.".to_string()),
        }
    };

    let raw = store.read_document_raw(&sc).map_err(|e| e.user_message())?;
    Ok(format!("## Reference Architecture: {}\n\n{}", sc, raw))
}

fn tool_evaluate_brownfield(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let language = get_str(args, "language")?;
    let project_type = get_str(args, "project_type")?;

    let engine = CatalogQueryEngine::with_builtins();
    let all_entries = engine.all_entries();

    // Verify the requested language/project_type exists
    if engine.find_exact(language, project_type).is_none() {
        return Ok(format!(
            "No catalog entry found for language='{}', project_type='{}'. \
            Use `list_catalog_languages` to see available options.",
            language, project_type
        ));
    }

    // Collect file paths from the project
    let mut file_paths = Vec::new();
    let project = Path::new(project_path);
    let pattern = project.join("**/*").display().to_string();
    if let Ok(entries) = glob::glob(&pattern) {
        for entry in entries.flatten() {
            if entry.is_file() {
                if let Ok(relative) = entry.strip_prefix(project) {
                    let path_str = relative.display().to_string();
                    // Skip hidden dirs and common noise
                    if !path_str.starts_with('.') && !path_str.contains("/node_modules/") && !path_str.contains("/target/") {
                        file_paths.push(path_str);
                    }
                }
            }
        }
    }

    let evaluator = BrownfieldEvaluator::new();
    let result = evaluator.evaluate(&file_paths, all_entries, format!("eval-{}-{}", language, project_type));

    let outcome_desc = match &result.outcome {
        ultra_metis_core::EvaluationOutcome::CatalogMatch { catalog_entry_id, .. } =>
            format!("Catalog Match ({})", catalog_entry_id),
        ultra_metis_core::EvaluationOutcome::DerivedArchitecture { .. } =>
            "Derived Architecture (no catalog match, good quality)".to_string(),
        ultra_metis_core::EvaluationOutcome::RecommendCatalogPattern { recommended_entry_id, .. } =>
            format!("Recommend Catalog Pattern ({})", recommended_entry_id),
        ultra_metis_core::EvaluationOutcome::RecordAsIs { .. } =>
            "Record As-Is".to_string(),
    };

    Ok(format!(
        "## Brownfield Evaluation\n\n\
        | Field | Value |\n\
        | ----- | ----- |\n\
        | Language | {} |\n\
        | Project Type | {} |\n\
        | Outcome | {} |\n\
        | Quality Score | {:.1}% |\n\
        | Files Analyzed | {} |",
        language,
        project_type,
        outcome_desc,
        result.quality_score,
        file_paths.len()
    ))
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ---------------------------------------------------------------------------
// Rules tool handlers
// ---------------------------------------------------------------------------

/// Helper to load all RulesConfig documents from the store
fn load_all_rules_configs(store: &DocumentStore) -> Result<Vec<RulesConfig>, String> {
    let docs = store
        .search_documents_with_options("rules_config", Some("rules_config"), None, false)
        .map_err(|e| e.user_message())?;

    let mut rules = Vec::new();
    for doc in &docs {
        let raw = store
            .read_document_raw(&doc.short_code)
            .map_err(|e| e.user_message())?;
        if let Ok(rc) = RulesConfig::from_content(&raw) {
            rules.push(rc);
        }
    }
    Ok(rules)
}

fn tool_query_rules(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let scope_str = get_optional_str(args, "scope");
    let protection_str = get_optional_str(args, "protection_level");
    let arch_ref = get_optional_str(args, "source_architecture_ref");
    let include_archived = args
        .get("include_archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let store = DocumentStore::new(Path::new(project_path));
    let rules_configs = load_all_rules_configs(&store)?;
    let refs: Vec<&RulesConfig> = rules_configs.iter().collect();
    let engine = RuleQueryEngine::new(&refs);

    let mut query = RuleQuery::new();
    if let Some(s) = scope_str {
        let scope = s.parse().map_err(|e: ultra_metis_core::DocumentValidationError| e.to_string())?;
        query = query.with_scope(scope);
    }
    if let Some(p) = protection_str {
        let level = p.parse().map_err(|e: ultra_metis_core::DocumentValidationError| e.to_string())?;
        query = query.with_protection_level(level);
    }
    if let Some(a) = arch_ref {
        query = query.with_source_architecture_ref(a);
    }
    if include_archived {
        query = query.including_archived();
    }

    let results = engine.query(&query);

    if results.is_empty() {
        return Ok("No rules matching the query.".to_string());
    }

    let mut output = format!(
        "## Rules Query Results ({})\n\n\
        | Short Code | Title | Scope | Protection | Phase |\n\
        | ---------- | ----- | ----- | ---------- | ----- |\n",
        results.len()
    );
    for rule in &results {
        let phase = rule
            .phase()
            .map(|p| p.to_string())
            .unwrap_or_else(|_| "?".to_string());
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            rule.metadata().short_code,
            rule.title(),
            rule.scope,
            rule.protection_level,
            phase
        ));
    }
    Ok(output)
}

fn tool_get_applicable_rules(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let target_scope_str = get_str(args, "target_scope")?;

    let target_scope: ultra_metis_core::domain::documents::rules_config::RuleScope =
        target_scope_str
            .parse()
            .map_err(|e: ultra_metis_core::DocumentValidationError| e.to_string())?;

    let store = DocumentStore::new(Path::new(project_path));
    let rules_configs = load_all_rules_configs(&store)?;
    let refs: Vec<&RulesConfig> = rules_configs.iter().collect();
    let engine = RuleQueryEngine::new(&refs);

    let results = engine.applicable_at_scope(target_scope);

    if results.is_empty() {
        return Ok(format!(
            "No rules applicable at scope '{}'.",
            target_scope_str
        ));
    }

    // Show inheritance chain
    let hierarchy: Vec<&str> = scope_hierarchy()
        .iter()
        .take_while(|s| {
            ultra_metis_core::domain::rules::query::scope_rank(**s)
                <= ultra_metis_core::domain::rules::query::scope_rank(target_scope)
        })
        .map(|s| match s {
            ultra_metis_core::domain::documents::rules_config::RuleScope::Platform => "platform",
            ultra_metis_core::domain::documents::rules_config::RuleScope::Organization => "org",
            ultra_metis_core::domain::documents::rules_config::RuleScope::Repository => "repo",
            ultra_metis_core::domain::documents::rules_config::RuleScope::Package => "package",
            ultra_metis_core::domain::documents::rules_config::RuleScope::Component => "component",
            ultra_metis_core::domain::documents::rules_config::RuleScope::Task => "task",
        })
        .collect();

    let mut output = format!(
        "## Applicable Rules at '{}' Scope ({} rules)\n\n\
        Inheritance chain: {}\n\n\
        | Short Code | Title | Scope | Protection |\n\
        | ---------- | ----- | ----- | ---------- |\n",
        target_scope_str,
        results.len(),
        hierarchy.join(" > ")
    );
    for rule in &results {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            rule.metadata().short_code,
            rule.title(),
            rule.scope,
            rule.protection_level
        ));
    }
    Ok(output)
}

fn tool_list_protected_rules(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;

    let store = DocumentStore::new(Path::new(project_path));
    let rules_configs = load_all_rules_configs(&store)?;
    let refs: Vec<&RulesConfig> = rules_configs.iter().collect();
    let engine = RuleQueryEngine::new(&refs);

    let results = engine.protected_rules();

    if results.is_empty() {
        return Ok("No protected rules found.".to_string());
    }

    let mut output = format!(
        "## Protected Rules ({})\n\n\
        | Short Code | Title | Scope | Source Architecture |\n\
        | ---------- | ----- | ----- | ------------------- |\n",
        results.len()
    );
    for rule in &results {
        let arch_ref = rule
            .source_architecture_ref
            .as_deref()
            .unwrap_or("-");
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            rule.metadata().short_code,
            rule.title(),
            rule.scope,
            arch_ref
        ));
    }
    Ok(output)
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
