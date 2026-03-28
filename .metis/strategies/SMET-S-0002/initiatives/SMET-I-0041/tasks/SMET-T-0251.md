---
id: dashboard-live-session-monitoring
level: task
title: "Dashboard Live Session Monitoring and Output Stream"
short_code: "SMET-T-0251"
created_at: 2026-03-28T00:37:00.452076+00:00
updated_at: 2026-03-28T01:02:10.374618+00:00
parent: SMET-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0041
---

# Dashboard Live Session Monitoring and Output Stream

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Build the dashboard's live session monitoring view: real-time output stream, enhanced session detail with event timeline, and multi-session list with urgency indicators.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Session detail page enhanced with a "Live Output" tab: connects to SSE endpoint, displays streaming events in a scrollable log view
- [ ] Output lines color-coded: info (default), warning (amber), error (red), summary (blue/bold)
- [ ] Auto-scroll to bottom on new events, with a "scroll lock" toggle to pause auto-scroll
- [ ] Event counter showing total events and rate (e.g., "142 events, ~3/s")
- [ ] Session detail "Timeline" tab: merged view of state changes, output summaries, approval events, and guidance injections in chronological order
- [ ] Multi-session list enhanced: urgency badges — red dot for `waiting_for_input`, amber for sessions with warnings, green pulse for actively running
- [ ] Session list auto-refreshes every 5 seconds
- [ ] SSE connection management: auto-reconnect on disconnect, show connection status indicator
- [ ] `api/events.ts` module with TypeScript types for session events and SSE connection helper
- [ ] TypeScript compiles cleanly

## Implementation Notes

### Technical Approach
- SSE in browser: use native `EventSource` API or a lightweight wrapper
- Event display: virtualized list for performance (or simple DOM append with max 1000 visible entries)
- Reconnect: on EventSource error, retry with exponential backoff (1s, 2s, 4s, max 30s)
- Output tab and Timeline tab as React tab components on the session detail page
- Urgency badges: derive from session state + recent events

### Dependencies
- SMET-T-0248 (SSE Stream endpoint)
- Existing session detail page from I-0040

## Status Updates

*To be added during implementation*