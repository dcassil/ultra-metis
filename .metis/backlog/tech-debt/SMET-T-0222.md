---
id: investigate-setup-cadre-decompose
level: task
title: "Investigate setup-cadre-decompose.sh drift from architecture repo"
short_code: "SMET-T-0222"
created_at: 2026-03-27T19:46:32.720691+00:00
updated_at: 2026-03-27T19:46:32.720691+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Investigate setup-cadre-decompose.sh drift from architecture repo

## Objective

Compare `plugins/cadre/scripts/setup-cadre-decompose.sh` against the architecture repo version to identify drift, and replace any remaining Metis references with Cadre equivalents.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Script may have diverged from the architecture repo and may contain leftover Metis naming
- **Benefits of Fixing**: Consistent naming and behavior aligned with architecture repo
- **Risk Assessment**: Low — script changes, but must verify decompose workflow still works

## Acceptance Criteria

- [ ] Diff against architecture repo version
- [ ] Document all differences
- [ ] Replace any Metis references with Cadre equivalents
- [ ] Verify decompose workflow still functions correctly

## Implementation Notes

### Files to Investigate
- `plugins/cadre/scripts/setup-cadre-decompose.sh` (this repo)
- Corresponding file in the architecture repo

## Status Updates

*To be added during implementation*