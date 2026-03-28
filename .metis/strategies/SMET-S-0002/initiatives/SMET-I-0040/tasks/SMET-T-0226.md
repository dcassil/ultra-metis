---
id: session-state-machine-and-control
level: task
title: "Session State Machine and Control Actions API"
short_code: "SMET-T-0226"
created_at: 2026-03-27T21:00:36.584572+00:00
updated_at: 2026-03-27T23:54:50.240504+00:00
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

# Session State Machine and Control Actions API

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Implement the session state machine validation logic and control action endpoints (stop, force-stop, pause, resume). The state machine enforces valid transitions and the control actions allow users to manage running sessions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] State transition validation function: `fn is_valid_transition(from: &SessionState, to: &SessionState) -> bool` enforcing allowed transitions: `starting→running`, `running→waiting_for_input`, `running→paused`, `running→completed`, `running→failed`, `running→stopped`, `waiting_for_input→running`, `paused→running`, `starting→failed`
- [ ] `POST /api/sessions/{id}/stop` — Validates session is `running` or `waiting_for_input`. Inserts `stop` command into `session_commands`. Returns 200.
- [ ] `POST /api/sessions/{id}/force-stop` — Inserts `force_stop` command. If `starting` with no process, transitions directly to `stopped`.
- [ ] `POST /api/sessions/{id}/pause` — Validates session is `running`. Inserts `pause` command. Returns 200.
- [ ] `POST /api/sessions/{id}/resume` — Validates session is `paused` or `waiting_for_input`. Inserts `resume` command. Returns 200.
- [ ] `POST /api/sessions/{id}/state` — Internal endpoint for runner to report state transitions. Validates transition is legal, updates session state, inserts `session_event`. Requires `MachineTokenAuth`.
- [ ] Invalid state transitions return 409 Conflict with descriptive error
- [ ] All control actions validate session belongs to current user
- [ ] Unit tests for state machine (all valid and invalid transitions) and each control endpoint

## Implementation Notes

### Technical Approach
- Control action endpoints don't change state directly — they queue commands for the runner. The runner executes the action and reports back via `POST /api/sessions/{id}/state`.
- Exception: `force-stop` on a `starting` session can transition directly since no process exists yet
- The `/state` endpoint uses `MachineTokenAuth` since it's called by the runner

### Dependencies
- SMET-T-0224 (Session Data Model), SMET-T-0225 (Session CRUD API)

## Status Updates

*To be added during implementation*