---
id: rename-rust-crate-directories-and
level: task
title: "Rename Rust crate directories and update all Cargo.toml files"
short_code: "SMET-T-0160"
created_at: 2026-03-23T20:14:25.350048+00:00
updated_at: 2026-03-23T20:19:34.763754+00:00
parent: SMET-I-0074
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0074
---

# Rename Rust crate directories and update all Cargo.toml files

## Parent Initiative
[[SMET-I-0074]]

## Objective
Rename all 8 Rust crate directories from `ultra-metis-*` to `cadre-*` using `git mv`, update the root Cargo.toml workspace members, update all individual crate Cargo.toml package names and internal dependencies, and update all Rust source files (`use` statements, string literals, module paths). Verify `cargo build` and `cargo test` pass afterward.

## Scope
- 8 crate directories: ultra-metis-core, ultra-metis-store, ultra-metis-mcp, ultra-metis-cli, ultra-metis-agents, ultra-metis-events, ultra-metis-notes, ultra-metis-policy
- Root Cargo.toml workspace members
- All crate Cargo.toml files (package names, dependency references)
- All .rs files with `use ultra_metis_*` or string literals containing "ultra-metis" or "ultra_metis"
- Benchmarks under benchmarks/practical/ that reference ultra-metis crates

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] All 8 crate directories renamed to cadre-*
- [ ] Root Cargo.toml references cadre-* workspace members
- [ ] All crate Cargo.toml files use cadre-* names
- [ ] All `use` statements reference cadre_* modules
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes

## Implementation Notes
1. `git mv` each crate directory
2. Find/replace in all Cargo.toml: `ultra-metis` → `cadre`
3. Find/replace in all .rs: `ultra_metis` → `cadre` (underscore form)
4. Find/replace string literals: `"ultra-metis` → `"cadre`
5. Build and test

## Status Updates