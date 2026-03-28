---
id: intervention-api-approval-response
level: task
title: "Intervention API: Approval Response and Guidance Injection"
short_code: "SMET-T-0249"
created_at: 2026-03-28T00:36:58.181970+00:00
updated_at: 2026-03-28T00:57:18.130544+00:00
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

# Intervention API: Approval Response and Guidance Injection

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Implement the Control Service endpoints for user intervention: responding to approval requests and injecting guidance into running sessions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `POST /api/sessions/{id}/respond` — Respond to pending approval. Body: `{approval_id, choice, note}`. Updates `pending_approvals` record (status=responded, response fields set). Queues a `respond` command to runner. DashboardAuth.
- [ ] `POST /api/sessions/{id}/inject` — Inject guidance. Body: `{message, injection_type}` where injection_type is `normal`/`side_note`/`interrupt`. Queues an `inject` command to runner with payload. Persists a `GuidanceInjected` event. DashboardAuth.
- [ ] Respond endpoint validates approval exists and is pending, returns 400 if already responded
- [ ] Respond endpoint changes session state from `waiting_for_input` back to `running` (if that was the state)
- [ ] Inject endpoint validates session is in `running` or `waiting_for_input` state
- [ ] Both endpoints validate session ownership
- [ ] Events for both actions persisted to `session_output_events`
- [ ] Unit tests for respond and inject operations

## Implementation Notes

### Technical Approach
- Both endpoints queue commands via `session_commands` (same mechanism as stop/pause/resume) — the runner picks them up on next poll
- `respond` command payload: `{approval_id, choice, note}`
- `inject` command payload: `{message, injection_type}`
- Respond also updates the `pending_approvals` row directly in the control service DB
- Events are also pushed to the SSE broadcast channel for live subscribers

### Dependencies
- SMET-T-0246 (Event Model), SMET-T-0248 (Event Ingestion/SSE)

## Status Updates

*To be added during implementation*