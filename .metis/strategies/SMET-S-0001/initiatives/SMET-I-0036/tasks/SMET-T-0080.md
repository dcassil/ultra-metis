---
id: parent-existence-and-hierarchy
level: task
title: "Parent existence and hierarchy validation in create_document"
short_code: "SMET-T-0080"
created_at: 2026-03-17T18:55:57.342677+00:00
updated_at: 2026-03-17T19:17:24.101990+00:00
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

# Parent existence and hierarchy validation in create_document

## Parent Initiative

[[SMET-I-0036]]

## Objective

Add parent document existence and hierarchy validation to `create_document` in the store layer. Currently, creating a document with a non-existent parent_id succeeds silently (SMET-T-0078), and the `HierarchyValidator` exists in the domain layer but is never called from the store. This task wires up both validations so invalid parent references are caught at creation time.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_document` with a non-existent `parent_id` returns `StoreError::Validation` with a message like "Parent document '{short_code}' not found"
- [ ] `create_document` with an invalid hierarchy (e.g., Task under Vision) returns `StoreError::Validation` with a message explaining valid parent types
- [ ] `create_document` with a valid parent succeeds as before (no regression)
- [ ] `HierarchyValidator::validate_parent()` is called from the store's `create_document` path
- [ ] Unit tests cover: non-existent parent, wrong parent type, correct parent type, no parent when allowed

## Implementation Notes

### Technical Approach
1. In `ultra-metis-store/src/store.rs` `create_document` method, before writing the file:
   - If `parent_id` is provided, call `read_document` (or check file existence) to verify the parent exists
   - Call `HierarchyValidator::validate_parent(child_type, parent_type)` to enforce hierarchy rules
2. Map `DocumentValidationError::InvalidParent` and `InvalidHierarchy` to appropriate `StoreError::Validation`
3. The original metis validates: parent exists, parent is correct type, parent is in correct phase (decompose/active for initiatives)

### Dependencies
- None — this is a standalone store-layer change

### Risk Considerations
- Must not break existing valid creation paths
- Parent phase validation (e.g., initiative must be in decompose/active to accept tasks) may be added here or in a separate task — scope to existence + type validation first

## Status Updates

### 2026-03-17
- Added `HierarchyValidator` import to store layer
- Added parent existence check in `create_document`: reads parent doc before creating child, returns clear error if parent doesn't exist
- Added `HierarchyValidator::validate_parent()` call after confirming parent exists, enforcing type-level hierarchy rules
- Updated `HierarchyValidator` to allow Initiative as valid Task parent (legacy/streamlined mode compatibility)
- Updated `valid_parent_types()` for Task to include Initiative
- Added 4 new store tests: nonexistent parent, wrong parent type, valid vision parent for initiative, vision rejects parent
- All 18 store tests pass
- Files modified: `store.rs`, `hierarchy.rs`