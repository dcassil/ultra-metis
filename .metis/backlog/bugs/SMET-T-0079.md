---
id: fix-transitioning-completed
level: task
title: "Fix: Transitioning completed initiative silently reports completed to completed"
short_code: "SMET-T-0079"
created_at: 2026-03-17T18:45:18.399833+00:00
updated_at: 2026-03-17T20:52:37.207227+00:00
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

# Fix: Transitioning completed initiative silently reports completed to completed

## Objective

When calling `transition_phase` on a document that is already in a terminal phase (e.g., `completed`), the store silently reports `completed -> completed` as a success instead of returning an error. Terminal phases should not allow any further transitions. The original metis plugin correctly errors on this case.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Affected Users**: All users transitioning documents
- **Reproduction Steps**:
  1. Initialize a project and create a vision document
  2. Transition the vision through `draft → review → published`
  3. Call `transition_phase` again on the published vision (or call it on a completed initiative)
  4. Observe: success response showing `completed -> completed` (or `published -> published`)
- **Expected vs Actual**:
  - **Expected**: Error returned: "Document {short_code} is in terminal phase 'completed' and cannot be transitioned further"
  - **Actual**: Silent success with `completed -> completed` output, no actual state change

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `transition_phase` returns an error when the document is in a terminal phase (`completed`, `published`, `superseded`, `decided` for ADRs)
- [ ] Error message clearly identifies the terminal phase and explains no further transitions are possible
- [ ] Documents in non-terminal phases still transition normally
- [ ] Forced transitions on terminal phases also error (terminal means terminal — force doesn't override finality)
- [ ] Unit test: transitioning a completed task returns error
- [ ] Unit test: transitioning a published vision returns error
- [ ] Unit test: transitioning an active task to completed still works

## Implementation Notes

### Technical Approach
- In `cadre-store`'s `transition_phase` method:
  1. After reading the document and parsing its current phase, check `DocumentType::valid_transitions_from(current_phase)`
  2. If the result is an empty vec, the phase is terminal — return error immediately
  3. Error message: "Document {short_code} is in terminal phase '{phase}' — no further transitions are possible"
- This check should happen BEFORE any target phase resolution or auto-advance logic

### Dependencies
- `cadre-store` crate
- `DocumentType::valid_transitions_from()` in `cadre-core`

## Status Updates

### 2026-03-17
- Investigated store.rs — fix was already implemented during SMET-I-0036
- `transition_phase` detects terminal phase at lines 470-476: if old_phase == new_phase, returns error "already in terminal phase"
- Tests `test_terminal_phase_vision_errors`, `test_terminal_phase_task_errors`, `test_terminal_phase_explicit_same_phase_errors` all pass
- No code changes needed — closing as already resolved