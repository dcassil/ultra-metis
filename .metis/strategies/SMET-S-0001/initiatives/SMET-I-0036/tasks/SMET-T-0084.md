---
id: error-message-quality-improvements
level: task
title: "Error message quality improvements across all operations"
short_code: "SMET-T-0084"
created_at: 2026-03-17T18:56:01.430967+00:00
updated_at: 2026-03-17T19:24:14.773036+00:00
parent: SMET-I-0036
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0036
---

# Error message quality improvements across all operations

## Parent Initiative

[[SMET-I-0036]]

## Objective

Audit and improve error messages across all cadre operations to be actionable, specific, and suggest fixes — matching or exceeding the original metis quality. The original metis maps errors to user-friendly messages with suggestions (e.g., "Document not found. Use list_documents to see available documents."). Cadre currently passes through raw error strings from the store layer.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every `StoreError` variant produces a user-friendly message with: what went wrong, why, and what to do instead
- [ ] `DocumentNotFound` errors suggest using `list_documents` to find valid short codes
- [ ] `InvalidPhaseTransition` errors include valid transitions from the current phase
- [ ] `Validation` errors for parent issues specify what parent types are valid
- [ ] `EditFailed` errors clarify the search text wasn't found and suggest checking with `read_document`
- [ ] Error messages are tested via snapshot tests or string assertions

## Implementation Notes

### Technical Approach
1. Add a `user_message()` method to `StoreError` that returns formatted, actionable error strings
2. In the MCP server layer, use `user_message()` instead of raw `.to_string()` when returning errors
3. Reference the original metis error messages as the quality bar — each should explain what failed, why, and suggest a fix
4. Key error message improvements:
   - `DocumentNotFound` → "Document '{code}' not found. Use list_documents to see available documents."
   - `InvalidPhaseTransition` → "Cannot transition from '{from}' to '{to}'. Valid next phases: {list}"
   - `InvalidDocumentType` → "Unknown document type '{type}'. Valid types: vision, initiative, task, adr"
   - `NotInitialized` → "No Metis workspace found at '{path}'. Run initialize_project first."

### Dependencies
- Should be done after T-0080, T-0081, T-0082, T-0083 so all error paths exist to improve

## Status Updates

### 2026-03-17
- Added `user_message()` method to `StoreError` with actionable guidance for each error variant
- Updated all MCP tool handlers to use `user_message()` instead of raw `to_string()`
- Key improvements: DocumentNotFound suggests list_documents, EditFailed suggests read_document, NotInitialized suggests initialize_project, InvalidDocumentType lists valid types
- Added 4 unit tests for user_message output quality
- All 29 tests pass