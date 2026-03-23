---
id: verify-cargo-build-and-tests-pass
level: task
title: "Verify cargo build and tests pass, then remove old cadre/ and metis/ directories"
short_code: "SMET-T-0095"
created_at: 2026-03-17T21:08:16.396559+00:00
updated_at: 2026-03-17T21:22:24.190981+00:00
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

# Verify cargo build and tests pass, then remove old cadre/ and metis/ directories

## Parent Initiative

[[SMET-I-0038]]

## Objective

Run `cargo build` and `cargo test` from the repo root to confirm the workspace compiles and all tests pass after the restructure. Then delete the now-empty `cadre/` directory and the retired `metis/` reference directory. Commit the completed restructure.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo build` exits 0 from repo root
- [ ] `cargo test` exits 0 from repo root with all tests passing
- [ ] `cadre/` directory does not exist (or is empty and deleted)
- [ ] `metis/` directory does not exist
- [ ] No broken symlinks or dangling references remain
- [ ] Final commit created with all restructure changes

## Implementation Notes

### Technical Approach
1. From repo root: `cargo build` — fix any compilation errors before proceeding
2. From repo root: `cargo test` — fix any test failures
3. Confirm `cadre/` is empty (crates and Cargo files were moved in prior tasks), then `rm -rf cadre/`
4. `rm -rf metis/` (retiring the old reference implementation)
5. Final `git add -A && git commit`

### Dependencies
- SMET-T-0091, SMET-T-0092, SMET-T-0093, SMET-T-0094 must all be complete

### Risk Considerations
- If `cargo build` fails, diagnose the error before deleting anything — the old directories may need to stay temporarily as reference
- Run `cargo test` before removing directories so failures can be diagnosed with full context

## Status Updates

### 2026-03-17
Pre-deletion audit:
- cadre/crates/ — empty (all 4 crates moved in T-0092) ✓
- cadre/Cargo.toml and Cargo.lock — removed in T-0093 ✓
- cadre/tests/e2e_test.sh → moved to tests/ (path resolution works: SCRIPT_DIR resolves to repo root) ✓
- cadre/benchmarks/ → moved to benchmarks/ at repo root ✓
- cadre/plugin.json → updated path from ${pluginDir}/../Cargo.toml to ${pluginDir}/Cargo.toml, moved to repo root ✓
- cadre/.mcp.json → moved to repo root in T-0094 ✓

Final verification:
- cargo build: 0 errors (23 pre-existing warnings only) ✓
- cargo test: 789 passed, 0 failed ✓
- rm -rf cadre/ and metis/ ✓
- Repo root layout: apps/ crates/ packages/ infra/ docs/ scripts/ tests/ benchmarks/ Cargo.toml Cargo.lock .mcp.json plugin.json ✓
✓ COMPLETE