//! MCP tool implementations
//!
//! Each tool maps to a DocumentStore operation.

use crate::protocol::ToolDefinition;
use serde_json::Value;
use std::path::Path;
use ultra_metis_store::DocumentStore;

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
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn tool_initialize_project(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let prefix = get_str(args, "prefix")?;

    let store = DocumentStore::new(Path::new(project_path));
    store
        .initialize(prefix)
        .map_err(|e| e.user_message())?;

    Ok(format!(
        "Project initialized at {} with prefix {}",
        project_path, prefix
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

    Ok(format!(
        "## Document Created\n\n{} created successfully\n\n| Field | Value |\n| ----- | ----- |\n| Title | {} |\n| Type | {} |\n| Short Code | {} |",
        short_code, title, doc_type, short_code
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

    let mut output = String::from("| Short Code | Title | Type | Phase |\n| ---------- | ----- | ---- | ----- |\n");
    for doc in &docs {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            doc.short_code, doc.title, doc.document_type, doc.phase
        ));
    }

    Ok(output)
}

fn tool_edit_document(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let search = get_str(args, "search")?;
    let replace = get_str(args, "replace")?;

    let store = DocumentStore::new(Path::new(project_path));
    store
        .edit_document(short_code, search, replace)
        .map_err(|e| e.user_message())?;

    Ok(format!("Document {} updated successfully", short_code))
}

fn tool_transition_phase(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let short_code = get_str(args, "short_code")?;
    let phase = get_optional_str(args, "phase");

    let store = DocumentStore::new(Path::new(project_path));
    let result = store
        .transition_phase(short_code, phase)
        .map_err(|e| e.user_message())?;

    Ok(format!(
        "## Phase Transition\n\n{}: {}",
        short_code, result
    ))
}

fn tool_search_documents(args: &Value) -> Result<String, String> {
    let project_path = get_str(args, "project_path")?;
    let query = get_str(args, "query")?;

    let store = DocumentStore::new(Path::new(project_path));
    let docs = store
        .search_documents(query)
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

    let store = DocumentStore::new(Path::new(project_path));
    store
        .archive_document(short_code)
        .map_err(|e| e.user_message())?;

    Ok(format!("Document {} archived", short_code))
}
