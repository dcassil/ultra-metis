---
id: final-validation-sweep-rg-check
level: task
title: "Final validation sweep: rg check, make build, make install, make test"
short_code: "SMET-T-0163"
created_at: 2026-03-23T20:15:10.438037+00:00
updated_at: 2026-03-23T20:24:47.157952+00:00
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

# Final validation sweep: rg check, make build, make install, make test

## Parent Initiative
[[SMET-I-0074]]

## Objective
Run a comprehensive validation sweep to confirm the rename is complete. Search for any remaining "cadre" or "cadre" references (excluding .metis/ archival content and .claude/worktrees/). Run `make build`, `make install`, and `make test` to verify everything compiles, installs, and passes tests. Fix any stragglers found.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] `rg "cadre"` returns zero hits outside .metis/ and .claude/worktrees/
- [ ] `make build` succeeds — produces cadre-mcp and cadre binaries
- [ ] `make install` copies binaries to ~/.local/bin
- [ ] `make test` passes with zero failures
- [ ] All initiative acceptance criteria verified

## Status Updates