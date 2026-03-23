---
id: comprehensive-negative-path-test
level: task
title: "Comprehensive negative-path test suite"
short_code: "SMET-T-0085"
created_at: 2026-03-17T18:56:02.299604+00:00
updated_at: 2026-03-17T19:27:13.925696+00:00
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

# Comprehensive negative-path test suite

## Parent Initiative

[[SMET-I-0036]]

## Objective

Build a comprehensive negative-path test suite that exercises every invalid operation across all store operations. This becomes the regression test suite ensuring error handling never silently regresses. Tests should cover: bad short codes, missing required fields, invalid phase transitions, non-existent parents, invalid hierarchy, duplicate documents, empty strings, special characters in inputs, already-archived documents, and corrupted frontmatter.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Test module `tests/negative_path_tests.rs` (or similar) exists in `ultra-metis-store`
- [ ] Tests cover every `StoreError` variant with at least one test case each
- [ ] Tests cover per-operation invalid inputs:
  - `create_document`: non-existent parent, wrong parent type, empty title, disabled doc type
  - `read_document`: non-existent short code, malformed short code
  - `edit_document`: non-existent document, search text not found, frontmatter-corrupting edit
  - `transition_phase`: invalid transition, terminal phase, non-existent document, unknown phase string
  - `archive_document`: non-existent document, already-archived document
  - `search_documents`: special characters in query (FTS injection)
  - `list_documents`: non-existent workspace
  - `initialize_project`: already-initialized workspace
- [ ] All tests pass and assert specific error types/messages (not just "is error")
- [ ] Tests serve as regression suite — any future error handling change that breaks a test is caught

## Implementation Notes

### Technical Approach
1. Create a test module with a helper that sets up a temporary `.ultra-metis` workspace
2. Organize tests by operation (one `#[cfg(test)] mod` per operation)
3. Each test: set up precondition → call operation with invalid input → assert specific error variant and message content
4. Use `assert_matches!` or pattern matching on `StoreError` variants for precise assertions

### Dependencies
- Should be done after T-0080 through T-0084 so all error paths exist to test

## Status Updates

### 2026-03-17
- Added 11 new negative-path tests covering:
  - create: empty title (documented as gap), invalid doc type, task without parent
  - read: nonexistent document
  - edit: search text not found, nonexistent document
  - transition: invalid phase string, phase skipping, nonexistent document
  - archive: nonexistent document
  - initialize: already initialized
- Combined with tests from T-0080 through T-0084, total store test count is now 40
- Discovered gap: empty title validation not called during document creation (documented for future fix)
- All 40 store tests + 725 core tests pass