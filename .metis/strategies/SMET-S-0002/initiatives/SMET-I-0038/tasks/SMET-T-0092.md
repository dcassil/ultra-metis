---
id: move-rust-crates-to-top-level
level: task
title: "Move Rust crates to top-level crates/ and rename super-metis-core to ultra-metis-core"
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

# Move Rust crates to top-level crates/ and rename super-metis-core to ultra-metis-core

## Parent Initiative

[[SMET-I-0038]]

## Objective

Move all four Rust crates from `super-metis/crates/` to the top-level `crates/` directory, rename the `super-metis-core` package to `ultra-metis-core`, and update all `Cargo.toml` files to reflect the new package name and internal dependency references.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `super-metis/crates/super-metis-core/` moved to `crates/ultra-metis-core/`
- [ ] `super-metis/crates/ultra-metis-cli/` moved to `crates/ultra-metis-cli/`
- [ ] `super-metis/crates/ultra-metis-mcp/` moved to `crates/ultra-metis-mcp/`
- [ ] `super-metis/crates/ultra-metis-store/` moved to `crates/ultra-metis-store/`
- [ ] `crates/ultra-metis-core/Cargo.toml` has `name = "ultra-metis-core"` (was `super-metis-core`)
- [ ] All crate `Cargo.toml` files that depend on `super-metis-core` updated to `ultra-metis-core`
- [ ] No references to `super-metis-core` remain in any `Cargo.toml`

## Implementation Notes

### Technical Approach
1. Use `mv` (or `cp` + delete) to move each crate directory from `super-metis/crates/` to `crates/`
2. In `crates/ultra-metis-core/Cargo.toml`, change `name = "super-metis-core"` to `name = "ultra-metis-core"`
3. In each dependent crate's `Cargo.toml`, update the dependency name from `super-metis-core` to `ultra-metis-core` and update the `path` to point to `../../ultra-metis-core` (relative to their new location)
4. Note: the workspace root Cargo.toml is handled in SMET-T-0093, so skip it here

### Dependencies
- SMET-T-0091 must be complete so that `crates/` directory exists

### Risk Considerations
- The rename from `super-metis-core` to `ultra-metis-core` affects any `use super_metis_core::` statements in Rust source — those also need updating (hyphen becomes underscore in Rust identifiers)

## Status Updates

### 2026-03-17
- Moved super-metis/crates/super-metis-core → crates/ultra-metis-core (already named ultra-metis-core from prior rename)
- Moved super-metis/crates/ultra-metis-cli → crates/ultra-metis-cli
- Moved super-metis/crates/ultra-metis-mcp → crates/ultra-metis-mcp
- Moved super-metis/crates/ultra-metis-store → crates/ultra-metis-store
- Fixed stale path in ultra-metis-store/Cargo.toml: `../super-metis-core` → `../ultra-metis-core`
- cli and mcp Cargo.toml paths were already correct (referenced ../ultra-metis-store)
- No super-metis-core name references found in .rs files (rename done in SMET-I-0033)
✓ COMPLETE