---
id: investigate-custom-loader-rs-drift
level: task
title: "Investigate custom_loader.rs drift from architecture repo"
short_code: "SMET-T-0217"
created_at: 2026-03-27T19:40:58.289179+00:00
updated_at: 2026-03-28T16:47:46.306475+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Investigate custom_loader.rs drift from architecture repo

## Objective

Compare `crates/cadre-core/src/domain/catalog/custom_loader.rs` in this repo against the architecture repo version to identify any drift, undocumented changes, or divergence that needs to be reconciled.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: `custom_loader.rs` may have diverged from the architecture repo — unclear if changes are intentional or accidental
- **Benefits of Fixing**: Ensures the two repos stay in sync or documents intentional differences
- **Risk Assessment**: Low risk if drift is intentional; medium risk if unintentional changes mask bugs

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Diff `custom_loader.rs` against the architecture repo version
- [ ] Document all differences found
- [ ] For each difference, determine if intentional or accidental
- [ ] Reconcile or document any drift

## Implementation Notes

### Files to Investigate
- `crates/cadre-core/src/domain/catalog/custom_loader.rs` (this repo)
- Corresponding file in the architecture repo

## Status Updates

*To be added during implementation*