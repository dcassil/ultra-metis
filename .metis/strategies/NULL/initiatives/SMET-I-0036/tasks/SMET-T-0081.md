---
id: terminal-phase-transition-and-auto
level: task
title: "Terminal phase transition and auto-advance error handling"
short_code: "SMET-T-0081"
created_at: 2026-03-17T18:55:58.213102+00:00
updated_at: 2026-03-17T19:19:39.768662+00:00
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

# Terminal phase transition and auto-advance error handling

## Parent Initiative

[[SMET-I-0036]]

## Objective

Fix terminal phase transition handling and auto-advance behavior (SMET-T-0079). Currently, transitioning a document in a terminal phase (e.g., Published, Completed) silently succeeds and returns the same phase — it should return an error. Also ensure auto-advance (no target phase specified) returns a clear error when there's no next phase. The original metis returns `"Invalid phase transition from {from} to {to}"` for these cases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `transition_phase` on a document in a terminal phase (Published, Completed, Superseded) returns `StoreError::Validation` with message "Document '{short_code}' is already in terminal phase '{phase}'. No further transitions are possible."
- [ ] Auto-advance (no target phase) on a terminal document returns the same error instead of silently returning the current phase
- [ ] Explicit transition to the same terminal phase also errors (e.g., Completed → Completed)
- [ ] Valid transitions continue to work (no regression)
- [ ] Unit tests cover: terminal Vision (Published), terminal Initiative (Completed), terminal Task (Completed), terminal ADR (Superseded)

## Implementation Notes

### Technical Approach
1. In `next_phase_in_sequence()` for each document type, return `None` for terminal phases instead of returning the current phase
2. In the store's `transition_phase`, when auto-advancing and `next_phase_in_sequence()` returns `None`, return an error
3. In `can_transition_to()`, ensure terminal → terminal returns false
4. Add a `is_terminal_phase()` helper to `DocumentType` for clarity

### Dependencies
- None — standalone change to store + domain layer

## Status Updates

### 2026-03-17
- Added terminal phase detection in store's `transition_phase`: if old_phase == new_phase after transition, returns error
- Catches both auto-advance and explicit same-phase transitions on terminal documents
- Added 3 tests: terminal vision (Published), terminal task (Completed), explicit same-phase transition
- All 21 store tests pass