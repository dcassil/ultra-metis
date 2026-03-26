---
id: cadre-store-refactor-functions
level: task
title: "cadre-store: Refactor Functions Exceeding Structural Clippy Thresholds"
short_code: "SMET-T-0184"
created_at: 2026-03-26T19:20:19.284023+00:00
updated_at: 2026-03-26T21:04:53.928174+00:00
parent: SMET-I-0091
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0091
---

# cadre-store: Refactor Functions Exceeding Structural Clippy Thresholds

## Parent Initiative

[[SMET-I-0091]]

## Objective

Refactor all functions in the `cadre-store` crate that violate the structural clippy thresholds: `too_many_lines` (80), `too_many_arguments` (7), and `cognitive_complexity` (25). This crate has 3 violations total, all in a single file.

## Violations (3 total)

### too_many_lines (3 violations)

| File | Line | Function | Lines |
|------|------|----------|-------|
| `src/store.rs` | 408 | `parse_document` | 82/80 |
| `src/store.rs` | 494 | `create_document` | 222/80 |
| `src/store.rs` | 911 | `transition_phase_with_options` | 133/80 |

### too_many_arguments (0 violations)

None.

### cognitive_complexity (0 violations)

None.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo clippy -p cadre-store --all-targets -- -D clippy::too_many_lines -D clippy::too_many_arguments -D clippy::cognitive_complexity` passes with zero warnings
- [ ] All existing tests pass (`cargo test -p cadre-store`)
- [ ] No public API changes

## Implementation Notes

### Technical Approach

All 3 violations are in `src/store.rs`:

- **`parse_document`** (82 lines): Barely over threshold. Extract document-type-specific parsing branches into helper functions.
- **`create_document`** (222 lines): The worst offender in this crate. This function handles creation for all document types. Extract each document type's creation logic into separate private methods (e.g., `create_vision`, `create_initiative`, `create_task`, etc.).
- **`transition_phase_with_options`** (133 lines): Extract validation logic and phase-specific side effects into helper functions.

### Dependencies
Depends on cadre-core. No blocking dependency on other tasks in this initiative.

## Status Updates

*To be added during implementation*