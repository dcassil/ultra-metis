---
id: fix-sessiondetailpage-crashes-on
level: task
title: "Fix: SessionDetailPage crashes on new session — undefined historicalEvents"
short_code: "SMET-T-0295"
created_at: 2026-03-29T03:02:44.256058+00:00
updated_at: 2026-03-29T16:40:22.131499+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Fix: SessionDetailPage crashes on new session — undefined historicalEvents

## Objective

Fix crash in SessionDetailPage when navigating to a newly created session. The API returns `{ events: undefined }` for sessions with no events yet, which sets `historicalEvents` to `undefined` and crashes the `useMemo` on line 84 when accessing `.length`.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users creating new sessions
- **Reproduction Steps**: 
  1. Create a new session and submit the prompt
  2. Screen goes blank
  3. Navigate back to the session — same blank screen
- **Expected vs Actual**: Page should render the session detail view. Instead, throws `Uncaught TypeError: Cannot read properties of undefined (reading 'length')` at SessionDetailPage.tsx:84.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] SessionDetailPage renders correctly for new sessions with no events
- [x] `historicalEvents` is always a valid array, never undefined
- [x] Existing sessions with events still work correctly

## Implementation Notes

### Root Cause
`getSessionEvents(id)` returns `resp.events` which is `undefined` when no events exist. Line 137 calls `setHistoricalEvents(resp.events)` without defaulting, so `historicalEvents` becomes `undefined`. The `useMemo` at line 83-86 then crashes accessing `historicalEvents.length`.

### Fix
1. Line 137: Default `resp.events` to `[]` — `setHistoricalEvents(resp.events ?? [])`
2. Line 84: Add defensive guard — `if (!historicalEvents || historicalEvents.length === 0)`

## Status Updates

- **2026-03-29**: Bug identified and fixed. Root cause was missing nullish coalescing on API response.