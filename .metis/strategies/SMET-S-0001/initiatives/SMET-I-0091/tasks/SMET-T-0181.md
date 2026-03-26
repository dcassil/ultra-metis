---
id: cadre-core-refactor-functions
level: task
title: "cadre-core: Refactor Functions Exceeding Structural Clippy Thresholds"
short_code: "SMET-T-0181"
created_at: 2026-03-26T19:20:04.281303+00:00
updated_at: 2026-03-26T19:20:04.281303+00:00
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

# cadre-core: Refactor Functions Exceeding Structural Clippy Thresholds

## Parent Initiative

[[SMET-I-0091]]

## Objective

Refactor all functions in the `cadre-core` crate that violate the structural clippy thresholds: `too_many_lines` (80), `too_many_arguments` (7), and `cognitive_complexity` (25). This is the largest crate with 27 violations total.

## Violations (27 total)

### too_many_lines (15 violations)

| File | Line | Function | Lines |
|------|------|----------|-------|
| `src/domain/bootstrap/init_flow.rs` | 214 | `build_summary` | 101/80 |
| `src/domain/catalog/brownfield_evaluator/rules_config_analyzer.rs` | 561 | `evaluate_jsts_quality` | 117/80 |
| `src/domain/catalog/brownfield_evaluator/rules_config_analyzer.rs` | 779 | `evaluate_python_quality` | 85/80 |
| `src/domain/catalog/brownfield_evaluator/rules_config_analyzer.rs` | 1030 | `evaluate_jsts_layering` | 105/80 |
| `src/domain/catalog/brownfield_evaluator/rules_config_analyzer.rs` | 1306 | `extract_import_linter_layers` | 84/80 |
| `src/domain/catalog/builtin_entries.rs` | 23 | `javascript_server` | 104/80 |
| `src/domain/catalog/builtin_entries.rs` | 131 | `javascript_react_app` | 94/80 |
| `src/domain/catalog/builtin_entries.rs` | 310 | `javascript_cli_tool` | 84/80 |
| `src/domain/documents/architecture/mod.rs` | 408 | `checklist_for_story_type` | 128/80 |
| `src/domain/documents/architecture/mod.rs` | 554 | `to_content` | 110/80 |
| `src/domain/documents/execution_record/mod.rs` | 340 | `from_content` | 96/80 |
| `src/domain/documents/execution_record/mod.rs` | 590 | `to_content` | 102/80 |
| `src/domain/operations/operation.rs` | 311 | `default_spec` | 106/80 |
| `src/domain/quality/parsers/coverage.rs` | 22 | `parse` | 108/80 |
| `src/domain/templates/mod.rs` | 135 | `new` | 95/80 |

### too_many_arguments (12 violations)

| File | Line | Function | Args |
|------|------|----------|------|
| `src/domain/documents/story/mod.rs` | 24 | `new` | 8/7 |
| `src/domain/documents/story/mod.rs` | 95 | `from_parts` | 11/7 |
| `src/domain/documents/quality_record/mod.rs` | 77 | `new_with_template` | 8/7 |
| `src/domain/documents/quality_record/mod.rs` | 124 | `from_parts` | 8/7 |
| `src/domain/documents/remediation_record/mod.rs` | 59 | `new` | 8/7 |
| `src/domain/documents/remediation_record/mod.rs` | 83 | `new_with_template` | 9/7 |
| `src/domain/documents/remediation_record/mod.rs` | 132 | `from_parts` | 9/7 |
| `src/domain/documents/rules_config/mod.rs` | 200 | `from_parts` | 8/7 |
| `src/domain/documents/validation_record/mod.rs` | 56 | `new` | 8/7 |
| `src/domain/documents/validation_record/mod.rs` | 80 | `new_with_template` | 9/7 |
| `src/domain/documents/validation_record/mod.rs` | 129 | `from_parts` | 9/7 |

### cognitive_complexity (1 violation)

| File | Line | Function | Complexity |
|------|------|----------|------------|
| `src/domain/catalog/brownfield_evaluator/rules_config_analyzer.rs` | 561 | `evaluate_jsts_quality` | 26/25 |

## Acceptance Criteria

- [ ] `cargo clippy -p cadre-core --all-targets -- -D clippy::too_many_lines -D clippy::too_many_arguments -D clippy::cognitive_complexity` passes with zero warnings
- [ ] All existing tests pass (`cargo test -p cadre-core`)
- [ ] No public API changes (refactoring is internal only, or parameter structs replace argument lists)

## Implementation Notes

### Technical Approach

- **too_many_lines**: Extract logical blocks into private helper functions. The `rules_config_analyzer.rs` file has 4 violations that can likely share common evaluation patterns. The `builtin_entries.rs` functions are catalog constructors that may benefit from builder helpers.
- **too_many_arguments**: Create `XxxParams` or `XxxOptions` structs for constructors in story, quality_record, remediation_record, rules_config, and validation_record modules. The `from_parts` and `new_with_template` patterns are repeated across multiple document types.
- **cognitive_complexity**: `evaluate_jsts_quality` (26/25) is barely over threshold; simplify with early returns or by extracting nested conditionals.

### Dependencies
None. cadre-core is the leaf dependency in the workspace.

## Status Updates

*To be added during implementation*