---
id: add-search-documents-filtering
level: task
title: "Add search_documents Filtering Parameters"
short_code: "SMET-T-0118"
created_at: 2026-03-18T04:10:23.390418+00:00
updated_at: 2026-03-18T04:18:04.499504+00:00
parent: SMET-I-0055
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0055
---

# Add search_documents Filtering Parameters

## Parent Initiative

[[SMET-I-0055]] - Tool Integration: Add Missing Tools and Parameter Support

## Objective

Add document_type, limit, and include_archived filtering parameters to the search_documents tool. Currently search returns all matching documents with no way to filter by type or cap result count.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] search_documents accepts optional `document_type` parameter (vision, initiative, task, adr, etc.)
- [ ] When document_type is provided, only documents of that type are returned
- [ ] search_documents accepts optional `limit` parameter (positive integer)
- [ ] When limit is provided, results are capped at that count
- [ ] search_documents accepts optional `include_archived` parameter (boolean, defaults to false)
- [ ] When include_archived is true, archived documents are included in results
- [ ] All parameters are optional — existing calls without them still work identically
- [ ] MCP tool schema updated in `crates/cadre-mcp/src/tools.rs`
- [ ] Unit tests for: type filter only, limit only, include_archived, combined filters
- [ ] All existing search tests still pass

## Implementation Notes

### Technical Approach
1. Add `SearchOptions` struct with `document_type: Option<DocumentType>`, `limit: Option<usize>`, `include_archived: Option<bool>`
2. Modify `search_documents` in store.rs to accept SearchOptions
3. After initial text match, apply type filter (compare document's level/type)
4. Apply archived filter (already partially handled but needs explicit parameter)
5. Apply limit via `.take(limit)` after filtering
6. Update MCP tool schema in tools.rs to include new optional parameters
7. Parse new parameters from JSON arguments in tool dispatch

### Key Files
- `crates/cadre-store/src/store.rs` - search_documents method
- `crates/cadre-mcp/src/tools.rs` - Tool schema and dispatch

### Dependencies
- None

## Status Updates

*To be added during implementation*