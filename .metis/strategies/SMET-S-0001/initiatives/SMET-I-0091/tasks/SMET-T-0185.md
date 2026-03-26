---
id: practical-benchmark-refactor
level: task
title: "practical-benchmark: Refactor Functions Exceeding Structural Clippy Thresholds"
short_code: "SMET-T-0185"
created_at: 2026-03-26T19:20:24.755198+00:00
updated_at: 2026-03-26T19:20:24.755198+00:00
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

# practical-benchmark: Refactor Functions Exceeding Structural Clippy Thresholds

## Parent Initiative

[[SMET-I-0091]]

## Objective

Refactor all functions in the `practical-benchmark` crate (`benchmarks/practical/`) that violate the structural clippy thresholds: `too_many_lines` (80), `too_many_arguments` (7), and `cognitive_complexity` (25). This crate has 14 violations total.

## Violations (14 total)

### too_many_lines (11 violations)

| File | Line | Function | Lines |
|------|------|----------|-------|
| `src/bin/run_benchmark.rs` | 12 | `main` | 98/80 |
| `src/gate_scorer.rs` | 39 | `score_initiative` | 89/80 |
| `src/gated_runner.rs` | 11 | `execute_with_gates` | 264/80 |
| `src/mcp_comparison.rs` | 72 | `run_tool_workflow` | 105/80 |
| `src/mcp_planning_comparison.rs` | 131 | `run_system_planning` | 83/80 |
| `src/reports.rs` | 105 | `format_report` | 134/80 |
| `src/reports.rs` | 266 | `format_run_detail_report` | 162/80 |
| `src/runner.rs` | 370 | `execute_autonomous` | 166/80 |
| `src/scoring.rs` | 410 | `make_test_run` (test) | 111/80 |
| `src/tool_comparison.rs` | 381 | `format_comparison_report` | 92/80 |
| `src/workspace.rs` | 42 | `setup` | 92/80 |

### too_many_arguments (0 violations)

None.

### cognitive_complexity (3 violations)

| File | Line | Function | Complexity |
|------|------|----------|------------|
| `src/gated_runner.rs` | 11 | `execute_with_gates` | 49/25 |
| `src/runner.rs` | 370 | `execute_autonomous` | 27/25 |
| `src/tool_comparison.rs` | 346 | `run_comparison` | 50/25 |

## Acceptance Criteria

- [ ] `cargo clippy -p practical-benchmark --all-targets -- -D clippy::too_many_lines -D clippy::too_many_arguments -D clippy::cognitive_complexity` passes with zero warnings
- [ ] All existing tests pass (`cargo test -p practical-benchmark`)
- [ ] No public API changes

## Implementation Notes

### Technical Approach

- **`gated_runner.rs` `execute_with_gates`** (264 lines, complexity 49): The worst function in the entire workspace. Break into phase-based helpers (setup, gate evaluation, execution, scoring). The high cognitive complexity suggests deeply nested conditionals that need flattening.
- **`runner.rs` `execute_autonomous`** (166 lines, complexity 27): Similar pattern to `execute_with_gates`. Extract phase logic into helpers.
- **`tool_comparison.rs` `run_comparison`** (complexity 50): Extract comparison logic for each tool category into separate functions.
- **`reports.rs`** (2 violations): Extract report section formatting into per-section helpers.
- **`scoring.rs` `make_test_run`** (test helper, 111 lines): Extract test fixture construction into builder helpers.
- **`bin/run_benchmark.rs` `main`** (98 lines): Extract subcommand dispatch into separate functions.
- **Remaining functions** (gate_scorer, mcp_comparison, mcp_planning_comparison, workspace): Minor overages (83-105 lines) -- extract logical blocks.

### Dependencies
Depends on cadre-core. No blocking dependency on other tasks in this initiative.

## Status Updates

*To be added during implementation*