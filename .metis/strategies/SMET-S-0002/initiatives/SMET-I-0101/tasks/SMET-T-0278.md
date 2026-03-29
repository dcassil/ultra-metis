---
id: sidebar-and-navigation-restructure
level: task
title: "Sidebar and Navigation Restructure: Remove Policies, Violations, Monitoring Pages"
short_code: "SMET-T-0278"
created_at: 2026-03-29T00:43:10.130095+00:00
updated_at: 2026-03-29T00:43:10.130095+00:00
parent: SMET-I-0101
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0101
---

# Sidebar and Navigation Restructure: Remove Policies, Violations, Monitoring Pages

Covers Initiative Issues 2 and 10. Major UI simplification.

## Objective

Restructure the sidebar navigation from 11 items (3 groups) to 7 items (2 groups), remove redundant pages (Policies, Violations, Monitoring), rename "Planning" to "Work", move History to the main group, and move notifications to a header bell icon.

## Acceptance Criteria

- [ ] Sidebar has exactly 7 nav items in 2 groups: Main (Machines, Sessions, History) and Work (Documents, Hierarchy, Governance)
- [ ] PoliciesPage.tsx is deleted and `/policies` route removed
- [ ] ViolationsPage.tsx is deleted and `/violations` route removed
- [ ] MonitoringPage.tsx is deleted and `/monitoring` route removed
- [ ] Notifications page removed from sidebar; notification bell icon added to Header with unread count badge
- [ ] "Planning" section renamed to "Work" in sidebar
- [ ] Default landing page changed from `/machines` to `/sessions`
- [ ] No broken links or 404s from remaining pages
- [ ] Dashboard compiles and renders without errors

## Implementation Notes

### Technical Approach

**Sidebar.tsx** — Restructure navigation arrays:
```
Before: navigation (2) + planningNavigation (4) + operationsNavigation (5) = 11
After:  mainNavigation (3) + workNavigation (3) = 6 + header bell = 7 total
```

Main navigation: Machines (ServerIcon, pending badge), Sessions (PlayIcon), History (ClockIcon)
Work navigation: Documents (DocumentTextIcon), Hierarchy (RectangleGroupIcon), Governance (ShieldCheckIcon)

**Header.tsx** — Add notification bell icon:
- Import `BellAlertIcon` and `useUnreadNotifications` hook
- Render bell icon with unread count badge (same styling as pending machine badge)
- Click navigates to `/notifications`

**App.tsx**:
- Remove route entries for `/policies`, `/violations`, `/monitoring`
- Remove imports for PoliciesPage, ViolationsPage, MonitoringPage
- Change default redirect: `<Navigate to="/sessions" replace />`
- Keep `/notifications` route (still accessible via header bell)

**Delete files**:
- `src/pages/PoliciesPage.tsx`
- `src/pages/ViolationsPage.tsx`  
- `src/pages/MonitoringPage.tsx`

### Files to Change
- `apps/control-dashboard/src/components/Sidebar.tsx`
- `apps/control-dashboard/src/components/Header.tsx`
- `apps/control-dashboard/src/App.tsx`

### Dependencies
None directly, but should be done before SMET-T-0284 (Governance page) since sidebar references it.

## Status Updates

*To be added during implementation*