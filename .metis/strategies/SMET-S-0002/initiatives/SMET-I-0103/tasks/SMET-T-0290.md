---
id: global-mobile-utilities-button
level: task
title: "Global Mobile Utilities: Button Stacking, Modal Sizing, Touch Targets, Tab Bar Scroll"
short_code: "SMET-T-0290"
created_at: 2026-03-29T02:21:09.148894+00:00
updated_at: 2026-03-29T02:28:15.664548+00:00
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

# Global Mobile Utilities: Button Stacking, Modal Sizing, Touch Targets, Tab Bar Scroll

## Parent Initiative

[[SMET-I-0103]] — Mobile-Responsive Dashboard

## Objective

Apply cross-cutting mobile improvements to shared UI components: button stacking, modal viewport sizing, touch target enforcement on small buttons, and horizontal-scrolling tab bars. These changes benefit all pages without page-specific work.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All button groups stack vertically below `sm` breakpoint (`flex-col sm:flex-row`, `w-full sm:w-auto` per button)
- [ ] Modal component: `max-w-[calc(100vw-2rem)] sm:max-w-md` so modals have 1rem margin on mobile
- [ ] Small buttons (`sm` size variant) have `min-h-[44px] min-w-[44px]` on mobile for touch targets
- [ ] Tab bar containers use `overflow-x-auto whitespace-nowrap` to scroll horizontally instead of wrapping
- [ ] Tab items have minimum `py-3 px-4` for touch-friendly targets
- [ ] No visual regressions on desktop (`lg`+)

## Implementation Notes

### Technical Approach

**Button stacking** — Find all instances of button groups (flex containers with multiple Button children) across pages and add `flex-col sm:flex-row gap-2` patterns. Individual buttons in groups get `w-full sm:w-auto`.

Key locations:
- `SessionDetailPage.tsx` — session control buttons (Stop, Pause, Resume)
- `MachineDetailPage.tsx` — Revoke, Remove buttons
- `NewSessionPage.tsx` — Cancel, Start buttons
- Any modal footer button groups

**Modal sizing** — Edit `src/components/ui/Modal.tsx`, change `max-w-md` to `max-w-[calc(100vw-2rem)] sm:max-w-md`.

**Touch targets** — Edit `src/components/ui/Button.tsx`, add `min-h-[44px]` to the `sm` size variant on mobile. Use `min-h-[44px] sm:min-h-0` so it only applies below `sm`.

**Tab bar scroll** — Create a shared pattern (or utility class) for tab navigation bars. Add `overflow-x-auto whitespace-nowrap -mx-4 px-4` (negative margin to allow edge-to-edge scroll) and `py-3 px-4` minimum on each tab item.

### Files to Change
- `src/components/ui/Modal.tsx` — viewport-aware max-width
- `src/components/ui/Button.tsx` — touch target minimum on sm variant
- `src/pages/SessionDetailPage.tsx` — button stacking, tab bar scroll
- `src/pages/MachineDetailPage.tsx` — button stacking, tab bar scroll
- `src/pages/NewSessionPage.tsx` — button stacking
- Any other pages with button groups or tab bars

### Dependencies
None — independent of other tasks.

## Status Updates

*To be added during implementation*