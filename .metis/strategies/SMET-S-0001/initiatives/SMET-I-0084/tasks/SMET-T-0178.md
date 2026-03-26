---
id: fix-existing-clippy-violations
level: task
title: "Fix Existing Clippy Violations Across Workspace"
short_code: "SMET-T-0178"
created_at: 2026-03-26T18:28:33.732613+00:00
updated_at: 2026-03-26T18:43:36.310358+00:00
parent: SMET-I-0084
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0084
---

# Fix Existing Clippy Violations Across Workspace

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0084]]

## Objective

Fix simple clippy violations that don't require structural refactoring — unused imports, unused variables, redundant closures, manual_strip, collapsible_if, etc. Structural issues (too_many_lines, too_many_arguments, cognitive_complexity) are deferred to SMET-I-0091. Bulk pedantic cleanup (use_self, uninlined_format_args, etc.) is deferred to SMET-I-0092.

## Acceptance Criteria

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with zero errors
- [ ] All unused imports and variables removed
- [ ] All auto-fixable default clippy violations resolved
- [ ] `cargo test --workspace` still passes
- [ ] Structural lints remain allowed in Cargo.toml — tracked by SMET-I-0091
- [ ] Bulk pedantic allows remain in Cargo.toml — tracked by SMET-I-0092

## Implementation Notes

### Violations in Scope (simple fixes)
- Unused imports (~20 in benchmarks/practical)
- Unused variables (~3)
- `manual_strip` (1) — use `strip_prefix()` instead
- `redundant_field_names` (1)
- `doc_link_with_quotes` (1)
- `future_not_send` (1)
- `collapsible_if` / `collapsible_else_if` (2)
- `redundant_clone` (2)

### Out of Scope (deferred)
- **SMET-I-0091**: too_many_lines (24), too_many_arguments (11), cognitive_complexity (4)
- **SMET-I-0092**: use_self (560), uninlined_format_args (373), and other bulk pedantic lints

### Technical Approach
1. Run `cargo clippy --fix` for auto-fixable issues
2. Manually fix remaining issues
3. Verify clean clippy + test suite

## Status Updates

*To be added during implementation*