---
id: monorepo-restructure-reorganize
level: initiative
title: "Monorepo Restructure: Reorganize into apps/crates/packages/infra/docs Layout"
short_code: "SMET-I-0038"
created_at: 2026-03-17T19:24:24.276014+00:00
updated_at: 2026-03-18T17:03:08.466750+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: monorepo-restructure-reorganize
---

# Monorepo Restructure: Reorganize into apps/crates/packages/infra/docs Layout Initiative

## Context

Ultra-metis is expanding from a single Rust workspace (`super-metis/`) into a full monorepo with web apps, shared TypeScript packages, infrastructure configs, and documentation. The current flat structure with `super-metis/` at the root won't scale as we add `control-web` (Next.js dashboard), `control-api` (API gateway), `machine-runner` (local daemon), and additional shared packages.

The `metis/` directory (old reference implementation) is being retired and should not be used.

## Goals & Non-Goals

**Goals:**
- Restructure the repo into the target monorepo layout: `apps/`, `crates/`, `packages/`, `infra/`, `docs/`, `scripts/`, `tests/`
- Move existing Rust crates from `super-metis/crates/` into top-level `crates/`
- Rename `super-metis-core` to `ultra-metis-core` for naming consistency
- Create placeholder directories for future components (apps, packages, infra, etc.)
- Preserve the Cargo workspace — update all paths in Cargo.toml files
- Ensure the project compiles and all tests pass after the move
- Remove or archive the `metis/` reference directory

**Non-Goals:**
- Creating the actual web apps, API, or daemon code (future initiatives)
- Adding TypeScript tooling, package.json, or monorepo manager (future initiatives)
- Changing any Rust code logic — this is purely a structural reorganization

## Current State

```
ultra-metis/
  metis/                        # old reference impl (retiring)
  super-metis/
    Cargo.toml                  # workspace root
    Cargo.lock
    crates/
      super-metis-core/         # domain logic (needs rename to ultra-metis-core)
      ultra-metis-cli/           # CLI
      ultra-metis-mcp/           # MCP integration
      ultra-metis-store/         # persistence
    target/
  .metis/                       # Metis project data (stays)
```

## Target State

```
ultra-metis/
  apps/
    control-web/                # (empty — future Next.js dashboard)
    control-api/                # (empty — future API gateway)
    machine-runner/             # (empty — future local daemon)
  crates/
    ultra-metis-core/           # moved + renamed from super-metis-core
    ultra-metis-store/          # moved from super-metis/crates/
    ultra-metis-cli/            # moved from super-metis/crates/
    ultra-metis-mcp/            # moved from super-metis/crates/
    ultra-metis-agents/         # (empty — future)
    ultra-metis-events/         # (empty — future)
    ultra-metis-notes/          # (empty — future)
    ultra-metis-policy/         # (empty — future)
  packages/
    shared-contracts/           # (empty — future ts schemas)
    ui/                         # (empty — future shared UI kit)
    config/                     # (empty — future shared configs)
  infra/
    docker/
    k8s/
    cloudflare/
    tailscale/
  docs/
    architecture/
    product/
    operations/
  scripts/
  tests/
  Cargo.toml                   # workspace root (moved up from super-metis/)
  Cargo.lock
  .metis/                      # unchanged
```

## Detailed Design

### Phase 1: Create directory structure
- Create all top-level directories and subdirectories
- Create placeholder `.gitkeep` files in empty future directories

### Phase 2: Move Rust crates
- Move `super-metis/crates/*` to `crates/`
- Rename `super-metis-core` to `ultra-metis-core`
- Update all internal `Cargo.toml` dependency paths and package names
- Move `super-metis/Cargo.toml` and `super-metis/Cargo.lock` to repo root
- Update workspace member paths

### Phase 3: Update references
- Update any hardcoded paths in source code
- Update `.mcp.json`, `plugin.json`, or other config files
- Update imports that reference the old `super-metis-core` package name

### Phase 4: Clean up
- Remove empty `super-metis/` directory
- Remove `metis/` reference directory
- Verify `cargo build` and `cargo test` pass

## Alternatives Considered

1. **Keep super-metis/ as a nested workspace** — rejected because it adds unnecessary nesting and the "super-metis" naming is being retired in favor of "ultra-metis"
2. **Use a JS monorepo tool (turborepo/nx) from the start** — rejected as premature; we can add this when we actually have JS/TS packages to manage
3. **Incremental moves (one crate at a time)** — rejected because a single atomic restructure is cleaner and avoids a prolonged mixed state

## Implementation Plan

1. Create directory scaffold (all dirs + .gitkeep for empty ones)
2. Move and rename Rust crates
3. Update Cargo workspace and dependency paths
4. Update all config files and source references
5. Verify build + tests
6. Remove old directories (super-metis/, metis/)
7. Final verification and commit

## Decomposition Summary

Initiative fully decomposed into 5 tasks:
- SMET-T-0091: Create monorepo directory scaffold (complete with details)
- SMET-T-0092: Move Rust crates and rename to ultra-metis-* (needs detailed AC and implementation notes)
- SMET-T-0093: Move Cargo workspace root and update members (needs detailed AC and implementation notes)
- SMET-T-0094: Update config files and source references (needs detailed AC and implementation notes)
- SMET-T-0095: Verify build/tests and remove old directories (needs detailed AC and implementation notes)

Tasks are ready for human review and population of remaining details before execution.

## Execution Summary (2026-03-17)

All 5 tasks completed successfully:
- SMET-T-0091 ✓ Directory scaffold created (apps/, crates/, packages/, infra/, docs/, scripts/, tests/ with .gitkeep)
- SMET-T-0092 ✓ Crates moved to crates/, stale path ../super-metis-core fixed in ultra-metis-store
- SMET-T-0093 ✓ Cargo.toml/Cargo.lock moved to repo root, workspace members updated
- SMET-T-0094 ✓ .mcp.json, plugin.json, CLAUDE.md all updated — no stale super-metis references remain
- SMET-T-0095 ✓ 789 tests pass, super-metis/ and metis/ removed, benchmarks/tests moved to repo root

Final repo layout matches target state. cargo build and cargo test both pass cleanly.