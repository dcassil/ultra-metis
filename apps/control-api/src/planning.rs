//! Planning data endpoints for the Control Dashboard.
//!
//! Exposes cadre document hierarchy, rules, and quality data
//! as read-only REST endpoints consumed by the dashboard frontend.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::PlanningState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Summary of a planning document for list/search responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningDocumentResponse {
    pub short_code: String,
    pub title: String,
    pub document_type: String,
    pub phase: String,
    pub parent_id: Option<String>,
    pub archived: bool,
}

/// Full document detail including raw content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDetailResponse {
    pub short_code: String,
    pub title: String,
    pub document_type: String,
    pub phase: String,
    pub parent_id: Option<String>,
    pub archived: bool,
    pub content: String,
    pub children: Vec<PlanningDocumentResponse>,
}

/// A node in the document hierarchy tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyNode {
    pub short_code: String,
    pub title: String,
    pub document_type: String,
    pub phase: String,
    pub children: Vec<HierarchyNode>,
}

/// A planning rule summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResponse {
    pub name: String,
    pub scope: String,
    pub description: String,
    pub protection_level: String,
}

/// Quality record for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResponse {
    pub short_code: String,
    pub title: String,
    pub gate_status: String,
    pub details: String,
}

/// Error body matching the existing API pattern.
#[derive(Debug, Serialize)]
struct PlanningError {
    error: String,
}

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub document_type: Option<String>,
    pub phase: Option<String>,
    pub parent_id: Option<String>,
    pub include_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub document_type: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RulesQuery {
    pub scope: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/planning/documents
// ---------------------------------------------------------------------------

pub async fn list_documents(
    State(state): State<PlanningState>,
    Query(params): Query<ListDocumentsQuery>,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(PlanningError {
                    error: "planning data not configured (CADRE_PROJECT_PATH not set)".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    };

    let include_archived = params.include_archived.unwrap_or(false);

    let docs = match store.list_documents_with_options(include_archived, params.parent_id.as_deref())
    {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(PlanningError {
                        error: format!("failed to list documents: {e}"),
                    })
                    .unwrap_or_default(),
                ),
            )
                .into_response();
        }
    };

    let mut results: Vec<PlanningDocumentResponse> = docs
        .into_iter()
        .map(|d| PlanningDocumentResponse {
            short_code: d.short_code,
            title: d.title,
            document_type: d.document_type,
            phase: d.phase,
            parent_id: d.parent_id,
            archived: d.archived,
        })
        .collect();

    // Apply optional type and phase filters
    if let Some(ref doc_type) = params.document_type {
        results.retain(|d| d.document_type == *doc_type);
    }
    if let Some(ref phase) = params.phase {
        results.retain(|d| d.phase == *phase);
    }

    (StatusCode::OK, Json(serde_json::to_value(results).unwrap_or_default())).into_response()
}

// ---------------------------------------------------------------------------
// GET /api/planning/documents/search
// ---------------------------------------------------------------------------

pub async fn search_documents(
    State(state): State<PlanningState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(PlanningError {
                    error: "planning data not configured".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    };

    let query = params.q.as_deref().unwrap_or("");
    let docs = match store.search_documents_with_options(
        query,
        params.document_type.as_deref(),
        params.limit,
        false,
    ) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(PlanningError {
                        error: format!("search failed: {e}"),
                    })
                    .unwrap_or_default(),
                ),
            )
                .into_response();
        }
    };

    let results: Vec<PlanningDocumentResponse> = docs
        .into_iter()
        .map(|d| PlanningDocumentResponse {
            short_code: d.short_code,
            title: d.title,
            document_type: d.document_type,
            phase: d.phase,
            parent_id: d.parent_id,
            archived: d.archived,
        })
        .collect();

    (StatusCode::OK, Json(serde_json::to_value(results).unwrap_or_default())).into_response()
}

// ---------------------------------------------------------------------------
// GET /api/planning/documents/:short_code
// ---------------------------------------------------------------------------

pub async fn get_document(
    State(state): State<PlanningState>,
    Path(short_code): Path<String>,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(PlanningError {
                    error: "planning data not configured".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    };

    // Read the parsed document for metadata
    let doc = match store.read_document(&short_code) {
        Ok(d) => d,
        Err(cadre_store::StoreError::DocumentNotFound { .. }) => {
            return (
                StatusCode::NOT_FOUND,
                Json(
                    serde_json::to_value(PlanningError {
                        error: format!("document '{short_code}' not found"),
                    })
                    .unwrap_or_default(),
                ),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(PlanningError {
                        error: format!("failed to read document: {e}"),
                    })
                    .unwrap_or_default(),
                ),
            )
                .into_response();
        }
    };

    // Read raw content for the body
    let raw_content = store.read_document_raw(&short_code).unwrap_or_default();

    // Find children
    let children = store
        .list_documents_with_options(false, Some(&short_code))
        .unwrap_or_default()
        .into_iter()
        .map(|d| PlanningDocumentResponse {
            short_code: d.short_code,
            title: d.title,
            document_type: d.document_type,
            phase: d.phase,
            parent_id: d.parent_id,
            archived: d.archived,
        })
        .collect();

    let summary = doc.to_summary();
    let phase = summary.phase;

    let detail = DocumentDetailResponse {
        short_code: summary.short_code,
        title: summary.title,
        document_type: summary.document_type,
        phase,
        parent_id: summary.parent_id,
        archived: summary.archived,
        content: raw_content,
        children,
    };

    (StatusCode::OK, Json(serde_json::to_value(detail).unwrap_or_default())).into_response()
}

// ---------------------------------------------------------------------------
// GET /api/planning/hierarchy
// ---------------------------------------------------------------------------

pub async fn get_hierarchy(State(state): State<PlanningState>) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(PlanningError {
                    error: "planning data not configured".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    };

    let docs = match store.list_documents(false) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(PlanningError {
                        error: format!("failed to load hierarchy: {e}"),
                    })
                    .unwrap_or_default(),
                ),
            )
                .into_response();
        }
    };

    // Build parent->children map
    let mut children_map: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    let mut root_indices: Vec<usize> = Vec::new();

    for (i, doc) in docs.iter().enumerate() {
        if let Some(ref parent) = doc.parent_id {
            children_map.entry(parent.clone()).or_default().push(i);
        } else {
            root_indices.push(i);
        }
    }

    fn build_tree(
        idx: usize,
        docs: &[cadre_store::store::DocumentSummary],
        children_map: &std::collections::HashMap<String, Vec<usize>>,
    ) -> HierarchyNode {
        let doc = &docs[idx];
        let child_nodes = children_map
            .get(&doc.short_code)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&ci| build_tree(ci, docs, children_map))
                    .collect()
            })
            .unwrap_or_default();

        HierarchyNode {
            short_code: doc.short_code.clone(),
            title: doc.title.clone(),
            document_type: doc.document_type.clone(),
            phase: doc.phase.clone(),
            children: child_nodes,
        }
    }

    let tree: Vec<HierarchyNode> = root_indices
        .iter()
        .map(|&i| build_tree(i, &docs, &children_map))
        .collect();

    (StatusCode::OK, Json(serde_json::to_value(tree).unwrap_or_default())).into_response()
}

// ---------------------------------------------------------------------------
// GET /api/planning/rules
// ---------------------------------------------------------------------------

pub async fn list_rules(
    State(state): State<PlanningState>,
    Query(params): Query<RulesQuery>,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(PlanningError {
                    error: "planning data not configured".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    };

    // Find all RulesConfig documents
    let docs = match store.search_documents_with_options("", Some("rules_config"), None, false) {
        Ok(d) => d,
        Err(_) => Vec::new(),
    };

    let mut rules: Vec<RuleResponse> = Vec::new();

    for doc_summary in &docs {
        if let Ok(raw) = store.read_document_raw(&doc_summary.short_code) {
            // Parse rules from the document content
            // Rules are stored as structured content; extract what we can
            rules.push(RuleResponse {
                name: doc_summary.title.clone(),
                scope: doc_summary
                    .parent_id
                    .clone()
                    .unwrap_or_else(|| "repo".into()),
                description: format!("Rules config: {}", doc_summary.short_code),
                protection_level: extract_protection_level(&raw),
            });
        }
    }

    // Apply scope filter
    if let Some(ref scope) = params.scope {
        rules.retain(|r| r.scope == *scope);
    }

    (StatusCode::OK, Json(serde_json::to_value(rules).unwrap_or_default())).into_response()
}

fn extract_protection_level(content: &str) -> String {
    if content.contains("protected") {
        "protected".to_string()
    } else if content.contains("advisory") {
        "advisory".to_string()
    } else {
        "standard".to_string()
    }
}

// ---------------------------------------------------------------------------
// GET /api/planning/quality/:short_code
// ---------------------------------------------------------------------------

pub async fn get_quality(
    State(state): State<PlanningState>,
    Path(short_code): Path<String>,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::to_value(PlanningError {
                    error: "planning data not configured".into(),
                })
                .unwrap_or_default()),
            )
                .into_response();
        }
    };

    // Verify the document exists
    if let Err(cadre_store::StoreError::DocumentNotFound { .. }) =
        store.read_document(&short_code)
    {
        return (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(PlanningError {
                    error: format!("document '{short_code}' not found"),
                })
                .unwrap_or_default(),
            ),
        )
            .into_response();
    }

    // Find quality records related to this document
    let quality_docs =
        match store.search_documents_with_options(&short_code, Some("quality_record"), None, false)
        {
            Ok(d) => d,
            Err(_) => Vec::new(),
        };

    let records: Vec<QualityResponse> = quality_docs
        .into_iter()
        .map(|d| QualityResponse {
            short_code: d.short_code,
            title: d.title,
            gate_status: if d.phase == "completed" {
                "pass".to_string()
            } else {
                "pending".to_string()
            },
            details: String::new(),
        })
        .collect();

    (StatusCode::OK, Json(serde_json::to_value(records).unwrap_or_default())).into_response()
}
