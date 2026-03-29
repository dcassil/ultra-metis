---
id: fix-mobile-button-overflow-in
level: task
title: "Fix mobile button overflow in modal footers, GuidanceInput, and remaining layout issues"
short_code: "SMET-T-0296"
created_at: 2026-03-29T03:03:22.336105+00:00
updated_at: 2026-03-29T03:06:07.154462+00:00
parent: SMET-I-0105
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0105
---

# Fix mobile button overflow in modal footers, GuidanceInput, and remaining layout issues

## Parent Initiative

[[SMET-I-0105]]

## Objective

Add `w-full sm:w-auto` to all modal footer buttons missing it, fix GuidanceInput injection type selector overflow, verify notification action buttons, and do a final sweep for any remaining overflow at 393px.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MachineDetailPage Revoke modal footer buttons have `w-full sm:w-auto`
- [ ] MachineDetailPage Remove modal footer buttons have `w-full sm:w-auto`
- [ ] PendingMachineCard Reject modal footer buttons have `w-full sm:w-auto`
- [ ] GuidanceInput Confirm Interrupt modal footer buttons have `w-full sm:w-auto`
- [ ] GuidanceInput injection type selector doesn't overflow on 393px
- [ ] Notification action buttons have proper mobile width constraints
- [ ] No horizontal overflow on any page at 393px viewport width
- [ ] Desktop layout unchanged
- [ ] TypeScript compiles cleanly

## Files to Edit

1. `apps/control-dashboard/src/pages/MachineDetailPage.tsx` — lines ~712, ~732
2. `apps/control-dashboard/src/components/PendingMachineCard.tsx` — line ~116
3. `apps/control-dashboard/src/components/GuidanceInput.tsx` — lines ~73, ~132
4. `apps/control-dashboard/src/pages/NotificationsPage.tsx` — line ~160
5. Any others found during audit

## Status Updates

**2026-03-28**: All fixes applied and verified:
- MachineDetailPage: Added `w-full sm:w-auto` to Revoke and Remove modal footer buttons (4 buttons)
- PendingMachineCard: Added `w-full sm:w-auto` to Reject modal footer buttons (2 buttons)
- GuidanceInput: Added `w-full sm:w-auto` to Confirm Interrupt modal footer buttons (2 buttons), added `overflow-x-auto` to injection type selector
- NotificationsPage: Verified OK — ghost buttons with `items-stretch` work correctly
- Full audit: All other modal footers and button containers already correct
- TypeScript compiles cleanly