---
id: machine-list-row-clickability-and
level: task
title: "Machine List Row Clickability and Status Display Enhancement"
short_code: "SMET-T-0277"
created_at: 2026-03-29T00:43:08.798012+00:00
updated_at: 2026-03-29T00:43:08.798012+00:00
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

# Machine List Row Clickability and Status Display Enhancement

Covers Initiative Issues 1 and 4. Unblocks Issues 3, 7, and 8.

## Objective

Make machine rows clickable in the machines list to navigate to the detail page, and enhance status visibility with colored dot indicators, a "Last Seen" column, and default sort by connectivity status.

## Acceptance Criteria

- [ ] Clicking a machine row in MachinesPage navigates to `/machines/{id}`
- [ ] Rows have visual hover affordance (cursor pointer, highlight) indicating clickability
- [ ] Colored dot indicator before machine name: green=online, yellow=stale, gray=offline
- [ ] "Last Seen" column displays relative time from `last_heartbeat` field
- [ ] Machine list is sorted by default: online first, then stale, then offline
- [ ] Machine detail page shows a prominent status banner (green for online, amber for stale, red for offline)

## Implementation Notes

### Technical Approach

**MachinesPage.tsx**:
1. Import `useNavigate` from react-router-dom
2. Add `onRowClick={(row) => navigate(`/machines/${row.id}`)}` to the `<Table>` component
3. Add a colored dot in the `name` column render function based on `connectivity_status`
4. Add a new column for "Last Seen" using `<RelativeTime timestamp={row.last_heartbeat} />`
5. Sort `activeMachines` by status priority: online (0) → stale (1) → offline (2) → unknown (3)

**MachineDetailPage.tsx**:
1. Add a status banner div above the tab navigation:
   - Online: green border-l-4 with "Machine is online" text
   - Stale: amber border-l-4 with "Machine hasn't sent a heartbeat recently"
   - Offline: red border-l-4 with "Machine is offline"

### Files to Change
- `apps/control-dashboard/src/pages/MachinesPage.tsx`
- `apps/control-dashboard/src/pages/MachineDetailPage.tsx`

### Dependencies
None — this is the first task and unblocks others.

## Status Updates

*To be added during implementation*