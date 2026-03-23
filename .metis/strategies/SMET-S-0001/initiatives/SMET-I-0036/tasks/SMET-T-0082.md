---
id: archive-cascading-and-already
level: task
title: "Archive cascading and already-archived detection"
short_code: "SMET-T-0082"
created_at: 2026-03-17T18:55:59.815101+00:00
updated_at: 2026-03-17T19:21:10.164066+00:00
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

# Archive cascading and already-archived detection

## Parent Initiative

[[SMET-I-0036]]

## Objective

Implement archive cascading (archiving a parent archives all children) and already-archived detection. Currently, `archive_document` on a parent initiative leaves child tasks active and orphaned, and archiving an already-archived document silently succeeds. The original metis archives entire initiative directories including all child tasks, and returns an error for already-archived documents.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `archive_document` on an initiative also archives all child tasks under that initiative
- [ ] `archive_document` on an already-archived document returns `StoreError::Validation` with message "Document '{short_code}' is already archived"
- [ ] After cascading archive, all child documents have `archived: true` in their frontmatter
- [ ] Archiving a standalone task (no children) still works as before
- [ ] Unit tests cover: cascade archive initiative with children, already-archived error, archive leaf task

## Implementation Notes

### Technical Approach
1. In `archive_document`, first check if `archived: true` already — return error if so
2. After marking the target document as archived, enumerate child documents:
   - For initiatives: find all tasks in the initiative's task directory
   - Mark each child as `archived: true`
3. The original metis uses the database to find children; ultra-metis can use the filesystem (tasks live under `initiatives/{SHORT_CODE}/tasks/`)

### Dependencies
- None — standalone store-layer change

## Status Updates

### 2026-03-17
- Added already-archived check: returns error if document is already archived
- Added cascade archiving: archives all child documents when archiving a parent
- Extracted `set_archived()` helper for single-document archive flag update
- Added 2 tests: already-archived error, cascade to children
- All 23 store tests pass