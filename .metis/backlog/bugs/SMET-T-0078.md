---
id: fix-allow-creating-tasks-with-non
level: task
title: "Fix: Allow creating tasks with non-existent parent IDs"
short_code: "SMET-T-0078"
created_at: 2026-03-17T18:45:17.559458+00:00
updated_at: 2026-03-17T20:52:36.818617+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Fix: Allow creating tasks with non-existent parent IDs

## Objective

The `ultra-metis-store` `create_document` operation accepts any string as `parent_id` without verifying the parent document actually exists. This means you can create a task under `FAKE-I-9999` and it silently succeeds, creating an orphaned document with a broken hierarchy. The store layer must validate that the parent document exists before creating a child document.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users creating child documents (tasks under initiatives, etc.)
- **Reproduction Steps**:
  1. Initialize a project with `ultra-metis-cli init /tmp/test`
  2. Run `ultra-metis-cli create /tmp/test task "My Task" --parent NONEXISTENT-I-9999`
  3. Observe: task is created successfully with no error
- **Expected vs Actual**:
  - **Expected**: Error returned: "Parent document NONEXISTENT-I-9999 not found"
  - **Actual**: Task created silently with broken parent reference

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_document` returns an error when `parent_id` is provided but no document with that short code exists in the project
- [ ] Error message clearly states: "Parent document {short_code} not found"
- [ ] Creating a document with a valid parent still works correctly
- [ ] Creating a document with no parent (e.g., vision) still works correctly
- [ ] Unit test: creating task with non-existent parent returns error
- [ ] Unit test: creating task with valid parent succeeds
- [ ] MCP server and CLI both surface the error to the user

## Implementation Notes

### Technical Approach
- In `ultra-metis-store/src/lib.rs` (or wherever `DocumentStore::create_document` is implemented):
  1. If `parent_id` is `Some(id)`, call `self.read_document(&id)` first
  2. If read returns not-found, return `Err("Parent document {id} not found")`
  3. If read succeeds, proceed with creation as normal
- The MCP server and CLI already propagate store errors, so this should surface automatically

### Dependencies
- `ultra-metis-store` crate

## Status Updates

### 2026-03-17
- Investigated store.rs — fix was already implemented during SMET-I-0036
- `create_document` validates parent existence at lines 282-293 and returns "Parent document not found" error
- Test `test_create_with_nonexistent_parent` already covers this case and passes
- No code changes needed — closing as already resolved