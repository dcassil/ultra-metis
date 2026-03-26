---
id: fix-existing-clippy-violations
level: task
title: "Fix Existing Clippy Violations Across Workspace"
short_code: "SMET-T-0178"
created_at: 2026-03-26T18:28:33.732613+00:00
updated_at: 2026-03-26T18:28:33.732613+00:00
parent: SMET-I-0084
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0084
---

# Fix Existing Clippy Violations Across Workspace

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0084]]

## Objective

Fix all existing clippy violations so the workspace passes `cargo clippy --workspace --all-targets -- -D warnings` cleanly. This includes the 66 current errors (mostly unused imports in `benchmarks/practical`) and any new pedantic/nursery warnings surfaced by SMET-T-0177's lint attributes.

## Acceptance Criteria

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with zero warnings/errors
- [ ] All unused imports removed (not just suppressed)
- [ ] `cargo test --workspace` still passes after fixes
- [ ] No `#![allow(...)]` annotations remain that were added as temporary suppressions in SMET-T-0177 (remove as violations are fixed)

## Implementation Notes

### Current Violations (66 errors)
- ~20 unused imports in `benchmarks/practical/src/` (`std::path::Path`, `chrono::Utc`, `Deserialize`, `Serialize`, `NamingConvention`, `TestPattern`, etc.)
- `stripping a prefix manually` — should use `strip_prefix()` instead
- `this if has identical blocks` — dead code or copy-paste error
- Additional pedantic/nursery warnings TBD after SMET-T-0177 adds lint attributes

### Technical Approach
1. Fix all unused imports first (bulk operation)
2. Fix `strip_prefix` manual pattern
3. Fix identical `if` blocks
4. Address pedantic/nursery violations introduced by SMET-T-0177
5. Remove temporary `#![allow(...)]` as violations are fixed
6. Verify full clippy + test suite passes

### Blocked By
- SMET-T-0177 (lint config must be in place first so we fix against the final lint rules)

## Status Updates

*To be added during implementation*