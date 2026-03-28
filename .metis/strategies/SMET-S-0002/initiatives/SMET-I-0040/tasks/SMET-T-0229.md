---
id: machine-runner-session-state
level: task
title: "Machine Runner Session State Reporting"
short_code: "SMET-T-0229"
created_at: 2026-03-27T21:00:39.256728+00:00
updated_at: 2026-03-28T00:10:15.988075+00:00
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

# Machine Runner Session State Reporting

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Implement the state reporting mechanism from the Machine Runner back to the Control Service. When a session's state changes locally, the runner reports it so the central state stays in sync.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Runner `client.rs` extended with `report_session_state(session_id, new_state, metadata)` — calls `POST /api/sessions/{id}/state`
- [ ] State reports for all lifecycle events: `starting→running`, `running→completed`, `running→failed`, `running→stopped`, `running→paused`, `paused→running`
- [ ] `waiting_for_input` detection: monitor stdout for approval prompts, report `running→waiting_for_input`
- [ ] Metadata JSON: `exit_code` (completion/failure), `error_message` (failure), `prompt_text` (waiting_for_input)
- [ ] Retry logic: queue report and retry on next heartbeat if control service unreachable
- [ ] Runner updates `started_at` on `running`, `completed_at` on terminal states
- [ ] Supervisor integrates with reporter: process exit triggers automatic state report
- [ ] Unit tests for state reporting client methods

## Implementation Notes

### Technical Approach
- Thin HTTP client layer calling `/state` endpoint from T-0226
- Supervisor emits state change events via channel; reporter task sends HTTP requests
- `waiting_for_input` detection: best-effort stdout scanning for known prompt patterns
- Failed reports stored in `Vec<PendingReport>`, retried on next heartbeat
- Same machine token auth as heartbeat/command requests

### Dependencies
- SMET-T-0226 (provides `/state` endpoint), SMET-T-0228 (generates state events)

## Status Updates

*To be added during implementation*