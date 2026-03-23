---
id: move-rust-crates-to-top-level
level: task
title: "Move Rust crates to top-level crates/ and rename cadre-core to cadre-core"
short_code: "SMET-T-0092"
created_at: 2026-03-17T21:08:12.727495+00:00
updated_at: 2026-03-17T21:12:32.686892+00:00
parent: SMET-I-0038
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0038
---

# Move Rust crates to top-level crates/ and rename cadre-core to cadre-core

## Parent Initiative

[[SMET-I-0038]]

## Objective

Move all four Rust crates from `cadre/crates/` to the top-level `crates/` directory, rename the `cadre-core` package to `cadre-core`, and update all `Cargo.toml` files to reflect the new package name and internal dependency references.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cadre/crates/cadre-core/` moved to `crates/cadre-core/`
- [ ] `cadre/crates/cadre-cli/` moved to `crates/cadre-cli/`
- [ ] `cadre/crates/cadre-mcp/` moved to `crates/cadre-mcp/`
- [ ] `cadre/crates/cadre-store/` moved to `crates/cadre-store/`
- [ ] `crates/cadre-core/Cargo.toml` has `name = "cadre-core"` (was `cadre-core`)
- [ ] All crate `Cargo.toml` files that depend on `cadre-core` updated to `cadre-core`
- [ ] No references to `cadre-core` remain in any `Cargo.toml`

## Implementation Notes

### Technical Approach
1. Use `mv` (or `cp` + delete) to move each crate directory from `cadre/crates/` to `crates/`
2. In `crates/cadre-core/Cargo.toml`, change `name = "cadre-core"` to `name = "cadre-core"`
3. In each dependent crate's `Cargo.toml`, update the dependency name from `cadre-core` to `cadre-core` and update the `path` to point to `../../cadre-core` (relative to their new location)
4. Note: the workspace root Cargo.toml is handled in SMET-T-0093, so skip it here

### Dependencies
- SMET-T-0091 must be complete so that `crates/` directory exists

### Risk Considerations
- The rename from `cadre-core` to `cadre-core` affects any `use cadre_core::` statements in Rust source — those also need updating (hyphen becomes underscore in Rust identifiers)

## Status Updates

### 2026-03-17
- Moved cadre/crates/cadre-core → crates/cadre-core (already named cadre-core from prior rename)
- Moved cadre/crates/cadre-cli → crates/cadre-cli
- Moved cadre/crates/cadre-mcp → crates/cadre-mcp
- Moved cadre/crates/cadre-store → crates/cadre-store
- Fixed stale path in cadre-store/Cargo.toml: `../cadre-core` → `../cadre-core`
- cli and mcp Cargo.toml paths were already correct (referenced ../cadre-store)
- No cadre-core name references found in .rs files (rename done in SMET-I-0033)
✓ COMPLETE