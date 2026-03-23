---
id: rename-namespace-to-ultra-metis-to
level: initiative
title: "Rename Namespace to ultra-metis to Avoid Conflicts with Existing Metis Plugin"
short_code: "SMET-I-0033"
created_at: 2026-03-17T02:59:09.866195+00:00
updated_at: 2026-03-17T03:06:42.384737+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: ultra-metis-core-engine-repo
initiative_id: rename-namespace-to-ultra-metis-to
---

# Rename Namespace to ultra-metis to Avoid Conflicts with Existing Metis Plugin Initiative

## Context

The project is currently named `super-metis-core` as a Rust crate. When this becomes a Claude Code plugin/MCP server, it will need a distinct namespace from the existing `metis` plugin (colliery-io/metis) that's already installed and actively used for work management. Both will coexist on the same machine — the existing metis handles current Metis Flight Levels, while ultra-metis/super-metis is the next-generation system.

Current naming:
- Repo directory: `ultra-metis/`
- Rust workspace: `super-metis/`
- Crate: `super-metis-core` (lib: `super_metis_core`)
- Metis prefix: `SMET`
- Various internal references use "super-metis" or "super_metis"

The naming needs to be unified under `ultra-metis` to match the repo name and avoid any confusion or namespace collision with the existing metis plugin.

## Goals & Non-Goals

**Goals:**
- Rename Rust crate from `super-metis-core` to `ultra-metis-core` (lib: `ultra_metis_core`)
- Rename workspace directory from `super-metis/` to `ultra-metis-core/` or keep as-is with updated Cargo.toml
- Update all `use super_metis_core::` imports to `use ultra_metis_core::`
- Update all internal string references ("super-metis", "Super-Metis") to "ultra-metis" / "Ultra-Metis"
- Update test files and integration tests
- Ensure `cargo build` and `cargo test` pass after rename
- Future MCP server/plugin will register as `ultra-metis` not `metis` or `super-metis`

**Non-Goals:**
- Changing the Metis document prefix (SMET stays — it's embedded in all existing documents)
- Renaming the `.metis/` directory structure (that's the existing metis plugin's convention)
- Changing any external metis plugin behavior

## Detailed Design

### Rename Steps
1. Update `super-metis/crates/super-metis-core/Cargo.toml`: package name → `ultra-metis-core`
2. Update `super-metis/Cargo.toml` workspace if needed
3. Global find/replace: `super_metis_core` → `ultra_metis_core` in all `.rs` files
4. Global find/replace: `super-metis-core` → `ultra-metis-core` in all `Cargo.toml` files
5. Update any string literals referencing "super-metis" or "Super-Metis"
6. Optionally rename the `super-metis/` directory to `ultra-metis/` (or leave as workspace subdirectory)
7. Run `cargo build` and `cargo test` to verify

### Risk: Low
This is a mechanical rename. No logic changes. All tests should pass unchanged after the rename.

## Alternatives Considered

1. **Keep super-metis name**: Rejected — repo is already called ultra-metis, and "super-metis" creates confusion about what the canonical name is.
2. **Use just "metis" with version suffix**: Rejected — too easy to conflict with the existing metis plugin.

## Implementation Plan

Phase 1: Rename crate and update all Cargo.toml files
Phase 2: Global rename in all Rust source files
Phase 3: Verify build and tests pass