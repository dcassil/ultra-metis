---
id: machine-detail-enrichment-sessions
level: task
title: "Machine Detail Enrichment: Sessions Tab, Violations Tab, and Cross-Navigation"
short_code: "SMET-T-0279"
created_at: 2026-03-29T00:43:11.498748+00:00
updated_at: 2026-03-29T00:57:28.750878+00:00
parent: SMET-I-0101
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0101
---

# Machine Detail Enrichment: Sessions Tab, Violations Tab, and Cross-Navigation

Covers Initiative Issues 8 (sessions as children of machines) and partial Issue 2 (violations tab).

## Objective

Enrich the Machine Detail page with a Sessions tab showing all sessions for that machine, a Violations tab showing policy violations for that machine, and add cross-navigation links between machines and sessions throughout the dashboard.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Machine Detail page has 4 tabs: Details | Sessions | Logs | Violations
- [ ] Sessions tab shows all sessions (active + historical) filtered by machine_id
- [ ] Sessions tab has state filter (all/running/completed/failed/stopped)
- [ ] Sessions tab rows are clickable, navigating to `/sessions/{id}`
- [ ] Violations tab shows policy violations for this machine (moved from standalone ViolationsPage)
- [ ] In global SessionsPage, the "Machine" column is a clickable link navigating to `/machines/{machine_id}`
- [ ] Machine link in SessionsPage stops event propagation so it doesn't trigger row click

## Implementation Notes

### Technical Approach

**MachineDetailPage.tsx**:
1. Expand `TabId` type to `'details' | 'sessions' | 'logs' | 'violations'`
2. Add Sessions tab content:
   - Use `listSessions({ machine_id: id })` to fetch sessions
   - Render a Table with columns: Title, State (with SessionStateBadge), Repo, Elapsed, Last Activity
   - Add state filter dropdown above the table
   - Add `onRowClick` to navigate to `/sessions/{id}`
3. Add Violations tab content:
   - Port the violations table from the deleted ViolationsPage
   - Use `listViolations({ machine_id: id })` or similar filtered endpoint
   - Show: timestamp, action, scope, reason, session link

**SessionsPage.tsx**:
1. Change the `machine_id` column to render a `<Link to={`/machines/${row.machine_id}`}>`
2. Add `onClick={(e) => e.stopPropagation()}` on the link to prevent triggering row navigation

### Files to Change
- `apps/control-dashboard/src/pages/MachineDetailPage.tsx`
- `apps/control-dashboard/src/pages/SessionsPage.tsx`

### Dependencies
- SMET-T-0277 (machine clickability) should be done first
- SMET-T-0278 (sidebar restructure) removes standalone ViolationsPage

## Status Updates

*To be added during implementation*