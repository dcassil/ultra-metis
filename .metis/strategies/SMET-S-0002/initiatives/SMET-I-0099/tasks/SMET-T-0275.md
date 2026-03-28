---
id: dashboard-machine-logs-tab
level: task
title: "Dashboard Machine Logs Tab"
short_code: "SMET-T-0275"
created_at: 2026-03-28T17:49:59.517223+00:00
updated_at: 2026-03-28T18:01:30.691295+00:00
parent: SMET-I-0099
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0099
---

# Dashboard Machine Logs Tab

## Parent Initiative

[[SMET-I-0099]] — Machine-Level Debug Log Pipeline

## Objective

Add a "Logs" tab to the machine detail page in the Control Dashboard with live SSE streaming and historical log query, reusing the LiveOutput component pattern.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Machine detail page gains a "Logs" tab (alongside existing content + Policy sections)
- [ ] Logs tab connects to SSE endpoint `GET /api/machines/{id}/logs/stream` for live streaming
- [ ] Log entries color-coded by level: DEBUG (gray), INFO (blue), WARN (amber), ERROR (red)
- [ ] Each entry shows: timestamp, level badge, target (module path), message
- [ ] Level filter dropdown: All, Debug, Info, Warn, Error — filters both live stream and historical query
- [ ] Toggle between "Live" mode (SSE) and "History" mode (paginated GET query)
- [ ] History mode: load older logs with "Load More" button, newest first
- [ ] Auto-scroll with scroll-lock toggle (same pattern as session LiveOutput)
- [ ] `api/machineLogs.ts` module with types and functions: `getMachineLogs()`, SSE connection helper
- [ ] TypeScript compiles cleanly

## Implementation Notes

### Technical Approach
- Reuse the `LiveOutput` component with a `mode` prop or create a `MachineLogViewer` variant
- SSE connection: same `EventSource` pattern as `useSessionEventStream` hook
- Level filter: apply client-side for live stream, pass as query param for history
- History pagination: `GET /api/machines/{id}/logs?limit=50&offset=0&level=warn`

### Dependencies
- SMET-T-0272 (API endpoints)
- Existing LiveOutput component and SSE patterns from I-0041

## Status Updates

*To be added during implementation*