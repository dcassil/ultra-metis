---
id: cadre-cli-refactor-functions
level: task
title: "cadre-cli: Refactor Functions Exceeding Structural Clippy Thresholds"
short_code: "SMET-T-0183"
created_at: 2026-03-26T19:20:15.021154+00:00
updated_at: 2026-03-26T19:20:15.021154+00:00
parent: SMET-I-0091
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0091
---

# cadre-cli: Refactor Functions Exceeding Structural Clippy Thresholds

## Parent Initiative

[[SMET-I-0091]]

## Objective

Refactor all functions in the `cadre-cli` crate that violate the structural clippy thresholds: `too_many_lines` (80), `too_many_arguments` (7), and `cognitive_complexity` (25). This crate has 3 violations total.

## Violations (3 total)

### too_many_lines (2 violations)

| File | Line | Function | Lines |
|------|------|----------|-------|
| `src/main.rs` | 516 | `main` | 185/80 |
| `src/main.rs` | 865 | `cmd_status` | 93/80 |

### too_many_arguments (1 violation)

| File | Line | Function | Args |
|------|------|----------|------|
| `src/main.rs` | 1465 | `cmd_notes_create` | 9/7 |

### cognitive_complexity (0 violations)

None.

## Acceptance Criteria

- [ ] `cargo clippy -p cadre-cli --all-targets -- -D clippy::too_many_lines -D clippy::too_many_arguments -D clippy::cognitive_complexity` passes with zero warnings
- [ ] All existing tests pass (`cargo test -p cadre-cli`)
- [ ] No public API changes

## Implementation Notes

### Technical Approach

- **main.rs `main`** (185 lines): This is the CLI entrypoint with clap subcommand dispatch. Extract each subcommand handler into its own function, leaving `main` as a thin dispatcher.
- **main.rs `cmd_status`** (93 lines): Extract formatting/display logic into helper functions.
- **main.rs `cmd_notes_create`** (9 args): Create a `NotesCreateParams` struct to bundle the arguments.

### Dependencies
Depends on cadre-core and cadre-store. No blocking dependency on other tasks in this initiative.

## Status Updates

*To be added during implementation*