---
id: create-rustfmt-toml-and-fix-all
level: task
title: "Create rustfmt.toml and Fix All Formatting Violations"
short_code: "SMET-T-0176"
created_at: 2026-03-26T18:27:57.750627+00:00
updated_at: 2026-03-26T18:31:39.376443+00:00
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

# Create rustfmt.toml and Fix All Formatting Violations

## Parent Initiative

[[SMET-I-0084]]

## Objective

Create a `rustfmt.toml` at the workspace root with stable-channel formatting rules and run `cargo fmt --all` to fix all 156 files that currently have formatting diffs.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `rustfmt.toml` exists at workspace root with stable-only options (max_width=100, tab_spaces=4, use_field_init_shorthand, use_try_shorthand, reorder_imports, reorder_modules)
- [ ] `imports_granularity` and `group_imports` are NOT included (nightly-only as of rustfmt 1.8.0-stable)
- [ ] `cargo fmt --all -- --check` passes with zero diffs
- [ ] `cargo test --workspace` still passes after formatting changes

## Implementation Notes

### Current State
- 156 files have formatting diffs against default rustfmt settings
- No rustfmt.toml exists today
- rustfmt version: 1.8.0-stable (2026-03-02)

### Technical Approach
1. Create `rustfmt.toml` with stable-only options at workspace root
2. Run `cargo fmt --all` to apply formatting
3. Verify `cargo fmt --all -- --check` returns 0
4. Verify `cargo test --workspace` still passes

### Key Decision
The initiative proposed `imports_granularity = "Module"` and `group_imports = "StdExternalCrate"` but both are nightly-only. Per the initiative's own guidance: "If stable doesn't support a feature, skip it rather than requiring nightly."

## Status Updates

*To be added during implementation*