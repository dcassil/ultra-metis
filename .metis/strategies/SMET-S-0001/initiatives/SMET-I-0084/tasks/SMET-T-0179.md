---
id: add-shellcheckrc-and-fix-shell
level: task
title: "Add .shellcheckrc and Fix Shell Script Violations"
short_code: "SMET-T-0179"
created_at: 2026-03-26T18:28:38.837914+00:00
updated_at: 2026-03-26T19:14:38.261146+00:00
parent: SMET-I-0084
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0084
---

# Add .shellcheckrc and Fix Shell Script Violations

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0084]]

## Objective

Create `.shellcheckrc` at workspace root and fix all shellcheck violations in the 7 Cadre plugin shell scripts (~812 LOC). Also fix any issues in the 3 repo-level scripts (`benchmarks/run-cadre-bench.sh`, `scripts/package.sh`, `tests/e2e_test.sh`). Vendor scripts under `vendor/` and `reference*/` are excluded.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.shellcheckrc` exists at workspace root with `shell=bash` default and `disable=SC1091` (can't follow non-constant source)
- [ ] `shellcheck plugins/cadre/**/*.sh` passes with zero warnings at severity=warning
- [ ] `shellcheck benchmarks/run-cadre-bench.sh scripts/package.sh tests/e2e_test.sh` passes
- [ ] All fixes are semantic preserving — no behavior changes
- [ ] shellcheck is installed locally via `brew install shellcheck` (or documented as requirement)

## Implementation Notes

### Scripts In Scope (Cadre plugin — 812 LOC)
| Script | Lines | Purpose |
|--------|-------|---------|
| hooks/block-todowrite.sh | 20 | Stop hook to block TodoWrite |
| hooks/pre-compact-hook.sh | 98 | Pre-compact context injection |
| hooks/session-start-hook.sh | 139 | Session start context injection |
| hooks/subagent-start-hook.sh | 57 | Subagent context injection |
| scripts/check-dependencies.sh | 70 | Dependency verification |
| scripts/setup-cadre-decompose.sh | 196 | Decompose loop setup |
| scripts/setup-cadre-ralph.sh | 232 | Ralph loop setup |

### Repo-Level Scripts (also in scope)
- `benchmarks/run-cadre-bench.sh`
- `benchmarks/run-practical-bench.sh`
- `scripts/package.sh`
- `tests/e2e_test.sh`

### Out of Scope
- `vendor/superpowers/**/*.sh` — vendored, not ours
- `reference - original metis/**/*.sh` — reference copy, not active

### Common shellcheck issues to expect
- SC2086: unquoted variables (word splitting risk)
- SC2034: unused variables
- SC2155: declare and assign separately
- SC2046: unquoted command substitution

### Blocked By
None — independent of Rust lint tasks

## Status Updates

*To be added during implementation*