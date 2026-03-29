---
id: fix-remaining-mobile-button
level: task
title: "Fix remaining mobile button stacking, GuidanceInput layout, modal footers, and filter controls"
short_code: "SMET-T-0294"
created_at: 2026-03-29T02:56:22.647934+00:00
updated_at: 2026-03-29T02:58:21.718320+00:00
parent: SMET-I-0104
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0104
---

# Fix remaining mobile button stacking, GuidanceInput layout, modal footers, and filter controls

## Parent Initiative

[[SMET-I-0104]]

## Objective

Fix all remaining mobile UI elements that don't respect screen width on iPhone 16 (393px). Apply the established responsive patterns from SMET-I-0103 to components that were missed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] PendingMachineCard: Approve/Reject buttons stack vertically on mobile
- [ ] PendingMachineCard: Card layout stacks content above buttons on mobile
- [ ] PendingMachineCard: Reject modal footer uses mobile stacking pattern
- [ ] GuidanceInput: Type selector stacks above input+send row on mobile
- [ ] GuidanceInput: Type selector buttons have 44px touch targets on mobile
- [ ] GuidanceInput: Interrupt confirm modal footer uses mobile stacking pattern
- [ ] ApprovalCard: Button container uses explicit flex-col sm:flex-row
- [ ] SessionsPage: Filter controls use grid layout on mobile
- [ ] All filter select elements are full-width on mobile
- [ ] Desktop layout unchanged (no regressions above sm breakpoint)
- [ ] Build passes with no errors

## Files to Modify

1. `PendingMachineCard.tsx` — Card layout, button group, modal footer
2. `GuidanceInput.tsx` — Layout restructure, touch targets, modal footer
3. `ApprovalCard.tsx` — Button container pattern
4. `SessionsPage.tsx` — Filter controls grid

## Status Updates

**2026-03-28**: All fixes applied and committed (97d42dd).
- PendingMachineCard: card layout stacks vertically, buttons stack with w-full, modal footer uses flex-col-reverse pattern
- GuidanceInput: type selector stacks above input+send row on mobile, buttons get min-h-[44px] touch targets, interrupt modal footer fixed
- ApprovalCard: flex-wrap replaced with flex-col sm:flex-row
- SessionsPage: filters converted from flex-wrap to grid-cols-2 sm:grid-cols-4 with w-full inputs
- TypeScript build passes clean