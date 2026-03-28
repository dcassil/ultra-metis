---
id: control-dashboard-session-list-and
level: task
title: "Control Dashboard Session List and Control UI"
short_code: "SMET-T-0231"
created_at: 2026-03-27T21:00:41.033349+00:00
updated_at: 2026-03-28T00:07:17.741474+00:00
parent: SMET-I-0040
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0040
---

# Control Dashboard Session List and Control UI

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Build the session list view and session detail page with state badges, elapsed time, and control action buttons.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Sessions page (`/sessions`): table with title, machine, repo, state badge, elapsed time, last activity, autonomy level
- [ ] State badges: `starting` (blue/pulse), `running` (green), `waiting_for_input` (amber), `paused` (gray), `completed` (green/check), `failed` (red), `stopped` (gray/x)
- [ ] "Waiting for input" sessions visually prominent
- [ ] Filterable by state, machine, repo. Sortable by created time, last activity, state
- [ ] Session detail (`/sessions/{id}`): full info, state timeline, elapsed counter, instructions, work item link
- [ ] Control buttons: "Stop" (running/waiting), "Force Stop" (any active), "Pause" (running), "Resume" (paused/waiting)
- [ ] Confirmation modal for "Force Stop"
- [ ] Auto-refresh every 10 seconds
- [ ] API functions: `stopSession()`, `forceStopSession()`, `pauseSession()`, `resumeSession()`
- [ ] Matches existing dashboard design patterns

## Implementation Notes

### Technical Approach
- Replaces stub `SessionsPage`
- Use Table, Badge, Button, Modal components
- Elapsed time: live-updating with `setInterval`
- Polling with `useEffect` + `setInterval` for auto-refresh
- Routes: `/sessions` → `SessionsPage`, `/sessions/:id` → `SessionDetailPage`

### Dependencies
- SMET-T-0225 (CRUD), SMET-T-0226 (Control Actions), SMET-T-0230 (shares API module)

## Status Updates

*To be added during implementation*