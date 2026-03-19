---
id: tool-integration-add-missing-tools
level: initiative
title: "Tool Integration: Add Missing Tools and Parameter Support"
short_code: "SMET-I-0055"
created_at: 2026-03-17T22:43:36.811872+00:00
updated_at: 2026-03-18T04:23:05.171078+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: tool-integration-add-missing-tools
---

# Tool Integration: Add Missing Tools and Parameter Support Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

Ultra-Metis MCP tools lack several capabilities present in the original Metis system:

**Missing Tools (2):**
1. `reassign_parent` - Move tasks between initiatives or to/from backlog
2. `index_code` - Index source code symbols for cross-reference with documents (tree-sitter based)

**Parameter Gaps (4 tools affected):**
1. `search_documents`: missing document_type filter, limit, include_archived
2. `edit_document`: missing replace_all parameter (only replaces first occurrence)
3. `transition_phase`: missing force parameter (cannot override validation checks)
4. `create_document`: missing decision_maker parameter for ADR documents

**Output Format Gaps:**
- Original Metis: Rich formatted output (tables, diffs, progress visualization, error context)
- Ultra-Metis: Minimal output (simple strings, success messages)

This reduces workflow efficiency and loses audit trail visibility when managing task reorganization, search filtering, and error recovery.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Implement `reassign_parent` tool for task reorganization (move between initiatives, to/from backlog)
- Implement `index_code` tool for source code symbol indexing and cross-referencing
- Add parameter support to existing tools (replace_all, force, document_type, decision_maker, include_archived)
- Enhance output formatting with rich tables, diffs, and contextual error messages
- Achieve feature parity with original Metis tool set (8/10 → 10/10 tools, all parameters supported)

**Non-Goals:**
- Custom search backends beyond FTS5 (SQLite upgrade handled separately in SMET-I-0057)
- Nested task reorganization (multi-level move operations)
- Streaming search results or pagination (MVP uses full result sets)

## Requirements

### Tool Implementation Checklist

**reassign_parent (Priority: Critical - blocks task workflow)**
- REQ-001: Move task from one initiative to another (validate parent is in decompose/active phase)
- REQ-002: Move task to backlog with category (bug/feature/tech-debt)
- REQ-003: Validate task is only type that can be reassigned
- REQ-004: Detect and report file conflicts at destination
- REQ-005: Update parent_id in task metadata

**index_code (Priority: High - advanced workflow)**
- REQ-006: Index source code symbols (functions, types, structs, traits) using tree-sitter
- REQ-007: Store indexed symbols with document cross-references
- REQ-008: Support multi-language indexing (Rust, JavaScript, Python)
- REQ-009: Query indexed symbols by name, type, or scope

**Parameter Enhancements (Priority: High - workflow improvements)**
- REQ-010: search_documents must support document_type filter
- REQ-011: search_documents must support limit parameter (e.g., limit=10)
- REQ-012: search_documents must support include_archived filter
- REQ-013: edit_document must support replace_all parameter (replace all occurrences)
- REQ-014: transition_phase must support force parameter (bypass validation)
- REQ-015: create_document must support decision_maker for ADR documents

**Output Formatting (Priority: Medium - UX improvement)**
- REQ-016: Tools must return formatted tables (markdown) not just strings
- REQ-017: edit_document must show diff visualization (before/after)
- REQ-018: Error messages must provide actionable context and next steps

## Detailed Design

### reassign_parent Tool
```rust
pub struct ReassignParentTool {
    pub project_path: String,
    pub short_code: String,
    pub new_parent_id: Option<String>,  // If None, move to backlog
    pub backlog_category: Option<String>,  // Required if new_parent_id is None
}

// Flow:
// 1. Load task, verify it's a task (not initiative/vision)
// 2. If new_parent_id: Load parent initiative, verify phase is decompose/active
// 3. Move file from old parent to new parent directory
// 4. Update parent_id in task metadata
// 5. Validate no filename conflicts at destination
// 6. Return success/conflict error
```

### index_code Tool
```rust
pub struct IndexCodeTool {
    pub project_path: String,
    pub patterns: Option<Vec<String>>,  // e.g., ["src/**/*.rs", "lib/**/*.js"]
    pub languages: Option<Vec<String>>,  // e.g., ["rust", "javascript"]
}

// Flow:
// 1. Discover source files matching patterns
// 2. For each file, use tree-sitter to extract symbols
// 3. Store index in .metis/code-index.db (SQLite)
// 4. Create cross-references linking symbols to documents
// 5. Return count of indexed symbols
```

### Parameter Enhancements
- Modify MCP tool signatures to include new parameters
- Update tool validation logic to handle new optional fields
- Maintain backward compatibility (old calls still work without new params)

## Testing Strategy

### Unit Tests
- reassign_parent: task type validation, parent phase validation, file conflict detection
- index_code: tree-sitter symbol extraction, multi-language support, database storage
- Parameter handling: all new parameters parsed and validated correctly
- Output formatting: tables render correctly, diffs show accurate before/after

### Integration Tests
- reassign_parent: full workflow from old to new parent, to backlog, and error cases
- index_code: end-to-end indexing with cross-reference creation
- Tool combinations: search with filters, edit with replace_all, transition with force

## Alternatives Considered

**Alternative 1: Keep reassign_parent as manual file operations**
- Rejected: Prone to errors; without formal tool, parent_id fields can get out of sync with directory structure

**Alternative 2: Delay index_code until full SQLite migration (SMET-I-0057)**
- Rejected: Code indexing is independent feature; can implement now without SQLite
- SQLite integration happens separately, index_code uses basic file storage initially

**Selected Approach: Implement all tools incrementally**
- reassign_parent and index_code as new MCP tools
- Parameter enhancements to existing tools (additive, backward compatible)
- Output formatting improvements (markdown tables, diffs)

## Implementation Plan

### Phase 1: reassign_parent Tool (Critical path)
- Implement ReassignParentTool struct with validation logic
- Add file system move operation (old parent → new parent directory)
- Add metadata update (parent_id field)
- Conflict detection and error handling
- MCP registration and integration
- Unit and integration tests

### Phase 2: Parameter Enhancements (High priority)
- Add replace_all to edit_document
- Add force to transition_phase
- Add document_type, limit, include_archived to search_documents
- Add decision_maker to create_document
- All changes backward compatible
- Unit tests for each parameter

### Phase 3: index_code Tool (Medium priority)
- Set up tree-sitter dependency (rust, javascript, python parsers)
- Implement symbol extraction for each language
- Create .metis/code-index.db (SQLite) for storing indexed symbols
- Implement search/query interface
- MCP registration
- Integration tests with multi-language codebases

### Phase 4: Output Formatting (Polish)
- Enhance tool response formatting (markdown tables)
- Add diff visualization to edit_document
- Improve error messages with actionable context
- Validation across all tools

### Exit Criteria
- reassign_parent tool fully functional and tested
- All parameter enhancements implemented and backward compatible
- index_code tool indexes code and creates cross-references
- Output formatting matches Metis quality
- All tools tested with realistic workflows
- Documentation updated