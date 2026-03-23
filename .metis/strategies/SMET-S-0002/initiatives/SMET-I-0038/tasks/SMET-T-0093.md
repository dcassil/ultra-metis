---
id: move-cargo-workspace-root-to-repo
level: task
title: "Move Cargo workspace root to repo root and update all member paths"
short_code: "SMET-T-0093"
created_at: 2026-03-17T21:08:13.889533+00:00
updated_at: 2026-03-17T21:13:21.177544+00:00
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

# Move Cargo workspace root to repo root and update all member paths

## Parent Initiative

[[SMET-I-0038]]

## Objective

Move `cadre/Cargo.toml` (the workspace root) and `cadre/Cargo.lock` to the repo root. Update the `[workspace]` members list to reflect that crates now live at `crates/<name>` instead of `crates/<name>`. Ensure `cargo metadata` resolves correctly from the repo root.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Cargo.toml` exists at repo root (moved from `cadre/Cargo.toml`)
- [ ] `Cargo.lock` exists at repo root (moved from `cadre/Cargo.lock`)
- [ ] Workspace `[workspace] members` list updated from `crates/<name>` to `crates/<name>` (paths are now relative to repo root)
- [ ] `cargo metadata --manifest-path Cargo.toml` runs without errors from repo root
- [ ] No `Cargo.toml` or `Cargo.lock` remains in `cadre/`

## Implementation Notes

### Technical Approach
1. Copy `cadre/Cargo.toml` to repo root
2. Update the `[workspace] members` array — old entries like `"crates/cadre-cli"` stay the same path string, but the root is now the repo root, so paths remain `"crates/<name>"`
3. Copy `cadre/Cargo.lock` to repo root
4. Delete `cadre/Cargo.toml` and `cadre/Cargo.lock`
5. Also check for any `[patch]` or `[replace]` sections that may reference old paths

### Dependencies
- SMET-T-0092 must be complete so that crates are already in their new locations

### Risk Considerations
- If any crate has a relative path dependency like `path = "../other-crate"`, those paths need updating since the nesting depth changed

## Status Updates

### 2026-03-17
- Created new Cargo.toml at repo root with updated members: crates/cadre-core, crates/cadre-store, crates/cadre-cli, crates/cadre-mcp
- Copied cadre/Cargo.lock to repo root
- Removed cadre/Cargo.toml and cadre/Cargo.lock
- `cargo metadata` from repo root resolves all 4 workspace members correctly
- Workspace root confirmed as /Users/danielcassil/projects/cadre
✓ COMPLETE