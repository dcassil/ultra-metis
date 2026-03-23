---
id: update-config-files-and-source
level: task
title: "Update config files and source references after monorepo restructure"
short_code: "SMET-T-0094"
created_at: 2026-03-17T21:08:15.148770+00:00
updated_at: 2026-03-17T21:15:10.654938+00:00
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

# Update config files and source references after monorepo restructure

## Parent Initiative

[[SMET-I-0038]]

## Objective

Update all non-Rust config and source files that contain hardcoded references to old paths (`cadre/`, `cadre-core`) or the old directory structure. This includes `.mcp.json`, `plugin.json`, `CLAUDE.md`, and any Rust source files that use `cadre-core` as a package name in `use` statements or feature flags.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.mcp.json` updated — any paths referencing `cadre/` updated to new locations
- [ ] `plugin.json` (if present) updated for new paths
- [ ] `CLAUDE.md` updated — references to `cadre/` paths corrected
- [ ] All Rust source files with `use cadre_core::` updated to `use cadre_core::`
- [ ] No remaining references to `cadre-core` or `cadre_core` in any `.rs` files
- [ ] No remaining references to `cadre/` paths in non-Rust config files (grep clean)

## Implementation Notes

### Technical Approach
1. `grep -r "cadre" --include="*.json" --include="*.md" --include="*.toml" .` to find all references (excluding `.metis/` and `target/`)
2. Update `.mcp.json` if it references binary paths like `cadre/target/...` or `cadre/Cargo.toml`
3. Update `CLAUDE.md` — the **Metis path** and **Build target** sections reference `cadre/`
4. `grep -r "cadre_core" --include="*.rs" .` to find Rust use statements and update to `cadre_core`
5. Check `scripts/` or any shell scripts for hardcoded paths

### Dependencies
- SMET-T-0092 and SMET-T-0093 must be complete

## Status Updates

### Completed (2026-03-17)

✓ Updated CLAUDE.md project context section to reference new crate locations
✓ Updated cadre/.mcp.json manifest path from cadre/Cargo.toml to Cargo.toml
✓ Updated cadre/plugin.json manifest path to ${pluginDir}/../Cargo.toml (points to repo root)
✓ Updated run-cadre-bench.sh build instructions comment
✓ Verified no remaining references to "cadre-core" or "cadre_core" in .rs files
✓ All configuration files now point to correct workspace locations