---
id: log-access-improvements-quick-link
level: task
title: "Log Access Improvements: Quick-Link from Machine List and Session Logs Tab"
short_code: "SMET-T-0283"
created_at: 2026-03-29T00:43:16.700814+00:00
updated_at: 2026-03-29T01:13:17.114949+00:00
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

# Log Access Improvements: Quick-Link from Machine List and Session Logs Tab

Covers Initiative Issue 7. Makes logs easily accessible from machine list and session detail.

## Objective

Add a quick-link logs icon on machine list rows for direct access to machine logs, and add a "Logs" tab to the SessionDetailPage showing machine-level debug logs filtered to the session's time range.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Machine list rows have a small log icon button that navigates directly to `/machines/{id}?tab=logs`
- [ ] MachineDetailPage respects `?tab=logs` URL parameter to auto-select the Logs tab on load
- [ ] SessionDetailPage has a 4th tab: "Logs" (after Overview, Live Output, Timeline)
- [ ] Session Logs tab shows `MachineLogViewer` filtered by the session's `started_at` to `completed_at` time range
- [ ] For active sessions, the Logs tab shows live machine logs (no end time filter)
- [ ] Log icon click on machine list stops event propagation (doesn't trigger row navigation)

## Implementation Notes

### Technical Approach

**MachinesPage.tsx**:
1. Add a column (or inline button in the name column) with a small log icon (DocumentTextIcon or similar)
2. On click: `navigate(`/machines/${row.id}?tab=logs`)` with `e.stopPropagation()`

**MachineDetailPage.tsx**:
1. Read `tab` from URL search params on mount: `const searchParams = new URLSearchParams(location.search)`
2. If `searchParams.get('tab') === 'logs'`, set initial `activeTab` to `'logs'`

**SessionDetailPage.tsx**:
1. Add `'logs'` to `TabId` type and `TABS` array
2. In the Logs tab content, render `<MachineLogViewer>` with:
   - `machineId={session.machine_id}`
   - Pass time range props (may need to enhance MachineLogViewer to accept `startTime`/`endTime` filter params)
3. For the time range filter, the API `GET /api/machines/{id}/logs` may need `?after=timestamp&before=timestamp` params

**MachineLogViewer.tsx** (potential enhancement):
- If it doesn't already support time range filtering, add optional `startTime` and `endTime` props
- Pass these as query params to the history fetch endpoint

### Files to Change
- `apps/control-dashboard/src/pages/MachinesPage.tsx`
- `apps/control-dashboard/src/pages/MachineDetailPage.tsx`
- `apps/control-dashboard/src/pages/SessionDetailPage.tsx`
- `apps/control-dashboard/src/components/MachineLogViewer.tsx` (if time range filter needed)

### Dependencies
- SMET-T-0277 (clickable machines) should be done first
- SMET-T-0279 (machine detail enrichment) adds the Sessions/Violations tabs — coordinate tab ordering

## Status Updates

*To be added during implementation*