---
id: table-component-card-mode-mobile
level: task
title: "Table Component Card Mode: Mobile Card Rendering Below sm Breakpoint"
short_code: "SMET-T-0288"
created_at: 2026-03-29T02:21:06.944738+00:00
updated_at: 2026-03-29T02:24:57.061744+00:00
parent: SMET-I-0103
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0103
---

# Table Component Card Mode: Mobile Card Rendering Below sm Breakpoint

## Parent Initiative

[[SMET-I-0103]] — Mobile-Responsive Dashboard

## Objective

Add a card-based mobile rendering path to the shared `<Table>` component so that below the `sm` (640px) breakpoint, table rows render as stacked cards instead of horizontal table rows. This is the foundational change that all list pages will use.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Table component renders cards below `sm` breakpoint, standard table at `sm`+
- [ ] Card layout: primary field as header, secondary fields as label:value pairs stacked vertically
- [ ] `onRowClick` handler preserved — entire card is tappable
- [ ] Cards have 12px gap between them, 16px horizontal padding
- [ ] Cards have minimum 44px touch target height
- [ ] Desktop table layout completely unchanged at `lg`+
- [ ] Table component accepts `mobileCardConfig` prop to define which columns map to card header/body/hidden
- [ ] TypeScript types are clean — no `any` escape hatches

## Implementation Notes

### Technical Approach

Modify `src/components/ui/Table.tsx`:

1. Add a `MobileCardConfig` type:
   - `headerColumn`: string — column key for card header
   - `badgeColumn?`: string — column key for inline badge in header
   - `bodyColumns`: string[] — column keys for label:value body pairs
   - `hiddenColumns?`: string[] — columns to omit on mobile entirely

2. Render two views in the same component:
   - `<div className="sm:hidden">` — card list view
   - `<div className="hidden sm:block">` — existing table view
   
3. Card structure:
   ```
   <div onClick={onRowClick} className="bg-white rounded-lg border p-4 cursor-pointer">
     <div className="flex items-center justify-between">
       {headerColumn value}
       {badgeColumn value}
     </div>
     <div className="mt-2 space-y-1 text-sm text-secondary-600">
       {bodyColumns as label: value pairs}
     </div>
   </div>
   ```

4. When no `mobileCardConfig` is provided, fall back to current `overflow-x-auto` table behavior (backwards compatible).

### Files to Change
- `src/components/ui/Table.tsx` — add card rendering path and types

### Dependencies
None — this is the foundation task.

## Status Updates

*To be added during implementation*