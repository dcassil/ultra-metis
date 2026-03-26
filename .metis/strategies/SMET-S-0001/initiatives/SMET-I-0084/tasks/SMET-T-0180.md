---
id: enhance-ci-workflow-and-makefile
level: task
title: "Enhance CI Workflow and Makefile with New Lint Targets"
short_code: "SMET-T-0180"
created_at: 2026-03-26T18:28:44.281782+00:00
updated_at: 2026-03-26T18:28:44.281782+00:00
parent: SMET-I-0084
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0084
---

# Enhance CI Workflow and Makefile with New Lint Targets

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0084]]

## Objective

Add a shellcheck CI job to `.github/workflows/ci.yml` and add `lint-shell` and `lint-strict` targets to the Makefile. Update the existing `ci` Make target to include the new lint targets so `make ci` runs the full quality suite locally.

## Acceptance Criteria

- [ ] CI workflow has a new `shellcheck` job that runs `shellcheck` on `plugins/cadre/` and repo-level scripts (excludes `vendor/` and `reference*/`)
- [ ] CI workflow's existing `clippy` job uses the new pedantic flags (or a separate `clippy-strict` job is added)
- [ ] Makefile has `lint-shell` target running shellcheck on in-scope scripts
- [ ] Makefile has `lint-strict` target running `cargo clippy` with pedantic warnings
- [ ] Makefile `ci` target updated: `ci: test lint lint-shell fmt-check`
- [ ] `make ci` passes locally after all prior tasks are complete
- [ ] CI workflow passes on a test push/PR

## Implementation Notes

### Current CI State
```yaml
jobs:
  test:     cargo test --workspace
  clippy:   cargo clippy --workspace --all-targets -- -D warnings
  fmt:      cargo fmt --all -- --check
```
No shellcheck job. Clippy only uses `-D warnings` (no pedantic).

### Current Makefile Targets
- `ci: test lint fmt-check` — `lint` is just `cargo clippy -- -D warnings`
- No `lint-shell` or `lint-strict` targets

### Technical Approach
1. Add `shellcheck` job to CI using `ludeeus/action-shellcheck@2.0.0` with scandir for `plugins/` and explicit file list for repo scripts
2. Update clippy CI job to include pedantic flags matching the lint attributes from SMET-T-0177
3. Add `lint-shell` Makefile target: `shellcheck plugins/cadre/**/*.sh plugins/cadre/hooks/*.sh benchmarks/*.sh scripts/*.sh tests/*.sh`
4. Add `lint-strict` Makefile target: `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
5. Update `ci` target to include new targets
6. Update `.PHONY` line

### Blocked By
- SMET-T-0176, SMET-T-0177, SMET-T-0178, SMET-T-0179 (all violations must be fixed first or CI will fail)

## Status Updates

*To be added during implementation*