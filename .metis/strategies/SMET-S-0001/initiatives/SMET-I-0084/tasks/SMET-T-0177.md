---
id: create-clippy-toml-and-add
level: task
title: "Create clippy.toml and Add Workspace Lint Attributes"
short_code: "SMET-T-0177"
created_at: 2026-03-26T18:28:28.462953+00:00
updated_at: 2026-03-26T18:28:28.462953+00:00
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

# Create clippy.toml and Add Workspace Lint Attributes

## Parent Initiative

[[SMET-I-0084]]

## Objective

Create `clippy.toml` at workspace root with complexity thresholds, and add `#![warn(clippy::pedantic)]` plus targeted lint attributes to each crate's `lib.rs`/`main.rs`. This task sets up the lint configuration only — it does NOT fix existing violations (that's SMET-T-0178).

## Acceptance Criteria

- [ ] `clippy.toml` exists at workspace root with thresholds: too-many-lines=80, too-many-arguments=7, type-complexity=250, cognitive-complexity=25
- [ ] Each crate's `lib.rs` or `main.rs` has `#![warn(clippy::pedantic)]` and `#![warn(clippy::nursery)]`
- [ ] Per-crate `#![allow(...)]` annotations added for known violations to keep the build green (documented as tech debt for SMET-I-0085 through SMET-I-0088)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` still passes (allowed violations suppress new errors)
- [ ] Key lints explicitly enabled: `too_many_lines`, `cognitive_complexity`, `large_enum_variant`, `wildcard_imports`

## Implementation Notes

### Current State
- Only `cadre-mcp` has any lint attributes (`#![allow(clippy::redundant_closure)]`, `#![allow(clippy::io_other_error)]`)
- No `clippy.toml` exists
- 66 clippy errors exist today (mostly unused imports in benchmarks/practical)

### Technical Approach
1. Create `clippy.toml` with threshold values
2. Add `#![warn(clippy::pedantic)]` and `#![warn(clippy::nursery)]` to each crate
3. Run `cargo clippy` to identify all new pedantic/nursery violations
4. Add targeted `#![allow(...)]` per crate for existing violations — each should have a comment like `// TODO: fix in SMET-I-0085`
5. Verify `cargo clippy --workspace --all-targets -- -D warnings` passes

### Blocked By
- SMET-T-0176 (formatting must be clean first so clippy operates on consistent code)

## Status Updates

*To be added during implementation*