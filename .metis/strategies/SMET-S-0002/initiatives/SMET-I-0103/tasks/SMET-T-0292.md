---
id: detail-pages-mobile-session
level: task
title: "Detail Pages Mobile: Session, Machine, Document, and Hierarchy Page Adjustments"
short_code: "SMET-T-0292"
created_at: 2026-03-29T02:21:11.294565+00:00
updated_at: 2026-03-29T02:31:15.628190+00:00
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

# Detail Pages Mobile: Session, Machine, Document, and Hierarchy Page Adjustments

## Parent Initiative

[[SMET-I-0103]] — Mobile-Responsive Dashboard

## Objective

Apply mobile-specific layout adjustments to all detail/single-item pages: Session Detail, Machine Detail, Document Detail, and Hierarchy Tree. Each page gets stacking, grid, and spacing adjustments for 393px width.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] **Session Detail**: metadata stacks vertically on mobile, tab bar scrolls horizontally
- [ ] **Machine Detail**: status banner stacks vertically (`flex-col sm:flex-row`), action buttons full-width stacked, tab bar scrolls horizontally
- [ ] **Document Detail**: metadata grid changes to `grid-cols-1 sm:grid-cols-2 md:grid-cols-3`, code blocks have `overflow-x-auto`
- [ ] **Hierarchy Tree**: indent reduced to `pl-3` on mobile (vs `pl-6` desktop), tree nodes have `py-3` minimum touch target, horizontal scroll for deep trees
- [ ] **Notifications Page**: action buttons stack vertically, 44px touch targets on dismiss/action buttons
- [ ] No horizontal scroll on any detail page at 393px width
- [ ] Desktop layouts unchanged

## Implementation Notes

### Technical Approach

**Session Detail** (`src/pages/SessionDetailPage.tsx`):
- Metadata section: `flex-col sm:flex-row` for key-value pairs
- Tab bar: already handled by SMET-T-0290 (tab bar scroll), just verify it works here

**Machine Detail** (`src/pages/MachineDetailPage.tsx`):
- Status banner at top: `flex-col sm:flex-row gap-2` for badge elements
- Action buttons (Revoke, Remove): `flex-col sm:flex-row` with `w-full sm:w-auto`
- Tab bar: verify horizontal scroll works

**Document Detail** (`src/pages/planning/DocumentDetailPage.tsx`):
- Metadata grid: change `grid-cols-2 sm:grid-cols-3` to `grid-cols-1 sm:grid-cols-2 md:grid-cols-3`
- Markdown content container: ensure `overflow-x-auto` on `<pre>` and `<code>` blocks

**Hierarchy Tree** (`src/pages/planning/HierarchyPage.tsx`):
- Tree node indent: `pl-3 sm:pl-6` per level
- Tree node touch target: `py-3` minimum padding
- Outer container: `overflow-x-auto` for deeply nested trees

**Notifications** (`src/pages/NotificationsPage.tsx`):
- Action buttons: `flex-col sm:flex-row`, each button `w-full sm:w-auto`
- Ensure dismiss buttons have 44px touch targets

### Files to Change
- `src/pages/SessionDetailPage.tsx`
- `src/pages/MachineDetailPage.tsx`
- `src/pages/planning/DocumentDetailPage.tsx`
- `src/pages/planning/HierarchyPage.tsx`
- `src/pages/NotificationsPage.tsx`

### Dependencies
- SMET-T-0290 (tab bar scroll utility) ideally done first, but can apply patterns inline.

## Status Updates

*To be added during implementation*