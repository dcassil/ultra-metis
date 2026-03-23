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

Cadre is expanding from a single Rust workspace (`cadre/`) into a full monorepo with web apps, shared TypeScript packages, infrastructure configs, and documentation. The current flat structure with `cadre/` at the root won't scale as we add `control-web` (Next.js dashboard), `control-api` (API gateway), `machine-runner` (local daemon), and additional shared packages.

The `metis/` directory (old reference implementation) is being retired and should not be used.

## Goals & Non-Goals

**Goals:**
- Restructure the repo into the target monorepo layout: `apps/`, `crates/`, `packages/`, `infra/`, `docs/`, `scripts/`, `tests/`
- Move existing Rust crates from `cadre/crates/` into top-level `crates/`
- Rename `cadre-core` to `cadre-core` for naming consistency
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
cadre/
  metis/                        # old reference impl (retiring)
  cadre/
    Cargo.toml                  # workspace root
    Cargo.lock
    crates/
      cadre-core/         # domain logic (needs rename to cadre-core)
      cadre-cli/           # CLI
      cadre-mcp/           # MCP integration
      cadre-store/         # persistence
    target/
  .metis/                       # Metis project data (stays)
```

## Target State

```
cadre/
  apps/
    control-web/                # (empty — future Next.js dashboard)
    control-api/                # (empty — future API gateway)
    machine-runner/             # (empty — future local daemon)
  crates/
    cadre-core/           # moved + renamed from cadre-core
    cadre-store/          # moved from cadre/crates/
    cadre-cli/            # moved from cadre/crates/
    cadre-mcp/            # moved from cadre/crates/
    cadre-agents/         # (empty — future)
    cadre-events/         # (empty — future)
    cadre-notes/          # (empty — future)
    cadre-policy/         # (empty — future)
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
  Cargo.toml                   # workspace root (moved up from cadre/)
  Cargo.lock
  .metis/                      # unchanged
```

## Detailed Design

### Phase 1: Create directory structure
- Create all top-level directories and subdirectories
- Create placeholder `.gitkeep` files in empty future directories

### Phase 2: Move Rust crates
- Move `cadre/crates/*` to `crates/`
- Rename `cadre-core` to `cadre-core`
- Update all internal `Cargo.toml` dependency paths and package names
- Move `cadre/Cargo.toml` and `cadre/Cargo.lock` to repo root
- Update workspace member paths

### Phase 3: Update references
- Update any hardcoded paths in source code
- Update `.mcp.json`, `plugin.json`, or other config files
- Update imports that reference the old `cadre-core` package name

### Phase 4: Clean up
- Remove empty `cadre/` directory
- Remove `metis/` reference directory
- Verify `cargo build` and `cargo test` pass

## Alternatives Considered

1. **Keep cadre/ as a nested workspace** — rejected because it adds unnecessary nesting and the "cadre" naming is being retired in favor of "cadre"
2. **Use a JS monorepo tool (turborepo/nx) from the start** — rejected as premature; we can add this when we actually have JS/TS packages to manage
3. **Incremental moves (one crate at a time)** — rejected because a single atomic restructure is cleaner and avoids a prolonged mixed state

## Implementation Plan

1. Create directory scaffold (all dirs + .gitkeep for empty ones)
2. Move and rename Rust crates
3. Update Cargo workspace and dependency paths
4. Update all config files and source references
5. Verify build + tests
6. Remove old directories (cadre/, metis/)
7. Final verification and commit

## Decomposition Summary

Initiative fully decomposed into 5 tasks:
- SMET-T-0091: Create monorepo directory scaffold (complete with details)
- SMET-T-0092: Move Rust crates and rename to cadre-* (needs detailed AC and implementation notes)
- SMET-T-0093: Move Cargo workspace root and update members (needs detailed AC and implementation notes)
- SMET-T-0094: Update config files and source references (needs detailed AC and implementation notes)
- SMET-T-0095: Verify build/tests and remove old directories (needs detailed AC and implementation notes)

Tasks are ready for human review and population of remaining details before execution.

## Execution Summary (2026-03-17)

All 5 tasks completed successfully:
- SMET-T-0091 ✓ Directory scaffold created (apps/, crates/, packages/, infra/, docs/, scripts/, tests/ with .gitkeep)
- SMET-T-0092 ✓ Crates moved to crates/, stale path ../cadre-core fixed in cadre-store
- SMET-T-0093 ✓ Cargo.toml/Cargo.lock moved to repo root, workspace members updated
- SMET-T-0094 ✓ .mcp.json, plugin.json, CLAUDE.md all updated — no stale cadre references remain
- SMET-T-0095 ✓ 789 tests pass, cadre/ and metis/ removed, benchmarks/tests moved to repo root

Final repo layout matches target state. cargo build and cargo test both pass cleanly.