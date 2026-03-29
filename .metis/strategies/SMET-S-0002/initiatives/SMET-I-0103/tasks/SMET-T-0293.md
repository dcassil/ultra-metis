---
id: mobile-verification-pass-393px
level: task
title: "Mobile Verification Pass: 393px Width Testing Across All Screens"
short_code: "SMET-T-0293"
created_at: 2026-03-29T02:21:12.298349+00:00
updated_at: 2026-03-29T02:34:24.080521+00:00
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

# Mobile Verification Pass: 393px Width Testing Across All Screens

## Parent Initiative

[[SMET-I-0103]] — Mobile-Responsive Dashboard

## Objective

Systematically test every screen in the dashboard at 393px viewport width (iPhone 16 portrait) and fix any remaining issues: horizontal scroll, clipped content, overlapping elements, unreachable buttons, or broken layouts.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every route tested at 393px width — no horizontal scroll on any page
- [ ] All interactive elements (buttons, links, cards, inputs) have 44px minimum touch target
- [ ] Approval flow works end-to-end on mobile: notification → session detail → approve/reject
- [ ] Guidance input flow works: session detail → type message → submit
- [ ] New session flow works: Start Session → select machine → enter prompt → submit
- [ ] Sidebar opens/closes correctly on mobile
- [ ] All card layouts render correctly with real data (not just empty states)
- [ ] No text truncation that hides critical information
- [ ] Document any remaining issues as backlog items if out of scope

## Implementation Notes

### Technical Approach

This is a manual verification pass through every route. Use browser DevTools responsive mode at 393px width.

**Routes to verify:**
1. `/sessions` — sessions list (cards)
2. `/sessions/new` — new session form
3. `/sessions/:id` — session detail (LiveOutput, tabs, guidance input, approval cards)
4. `/machines` — machines list (cards)
5. `/machines/:id` — machine detail (tabs, status banner, actions)
6. `/history` — history list (cards)
7. `/history/:id` — history detail
8. `/planning/documents` — documents list (cards)
9. `/planning/documents/:shortCode` — document detail (metadata grid, markdown content)
10. `/planning/hierarchy` — hierarchy tree
11. `/planning/governance` — governance tabbed view
12. `/notifications` — notifications list (cards)

**For each route, verify:**
- No horizontal scrollbar appears
- Content is readable without zooming
- All buttons/links are tappable (44px minimum)
- Cards render with correct field priority
- Navigation works (tap card → detail page → back)
- Modals open within viewport bounds

**Fix any issues found inline** — this task includes both testing and fixing.

### Dependencies
- All other tasks (T-0288 through T-0292) should be completed first.

## Status Updates

*To be added during implementation*