---
id: rename-namespace-to-cadre-to
level: initiative
title: "Rename Namespace to cadre to Avoid Conflicts with Existing Metis Plugin"
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
strategy_id: cadre-core-engine-repo
initiative_id: rename-namespace-to-cadre-to
---

# Rename Namespace to cadre to Avoid Conflicts with Existing Metis Plugin Initiative

## Context

The project is currently named `cadre-core` as a Rust crate. When this becomes a Claude Code plugin/MCP server, it will need a distinct namespace from the existing `metis` plugin (colliery-io/metis) that's already installed and actively used for work management. Both will coexist on the same machine — the existing metis handles current Metis Flight Levels, while cadre/cadre is the next-generation system.

Current naming:
- Repo directory: `cadre/`
- Rust workspace: `cadre/`
- Crate: `cadre-core` (lib: `cadre_core`)
- Metis prefix: `SMET`
- Various internal references use "cadre" or "cadre"

The naming needs to be unified under `cadre` to match the repo name and avoid any confusion or namespace collision with the existing metis plugin.

## Goals & Non-Goals

**Goals:**
- Rename Rust crate from `cadre-core` to `cadre-core` (lib: `cadre_core`)
- Rename workspace directory from `cadre/` to `cadre-core/` or keep as-is with updated Cargo.toml
- Update all `use cadre_core::` imports to `use cadre_core::`
- Update all internal string references ("cadre", "Cadre") to "cadre" / "Cadre"
- Update test files and integration tests
- Ensure `cargo build` and `cargo test` pass after rename
- Future MCP server/plugin will register as `cadre` not `metis` or `cadre`

**Non-Goals:**
- Changing the Metis document prefix (SMET stays — it's embedded in all existing documents)
- Renaming the `.metis/` directory structure (that's the existing metis plugin's convention)
- Changing any external metis plugin behavior

## Detailed Design

### Rename Steps
1. Update `cadre/crates/cadre-core/Cargo.toml`: package name → `cadre-core`
2. Update `cadre/Cargo.toml` workspace if needed
3. Global find/replace: `cadre_core` → `cadre_core` in all `.rs` files
4. Global find/replace: `cadre-core` → `cadre-core` in all `Cargo.toml` files
5. Update any string literals referencing "cadre" or "Cadre"
6. Optionally rename the `cadre/` directory to `cadre/` (or leave as workspace subdirectory)
7. Run `cargo build` and `cargo test` to verify

### Risk: Low
This is a mechanical rename. No logic changes. All tests should pass unchanged after the rename.

## Alternatives Considered

1. **Keep cadre name**: Rejected — repo is already called cadre, and "cadre" creates confusion about what the canonical name is.
2. **Use just "metis" with version suffix**: Rejected — too easy to conflict with the existing metis plugin.

## Implementation Plan

Phase 1: Rename crate and update all Cargo.toml files
Phase 2: Global rename in all Rust source files
Phase 3: Verify build and tests pass