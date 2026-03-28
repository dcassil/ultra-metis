---
id: integration-tests-for-live
level: task
title: "Integration Tests for Live Monitoring and Intervention"
short_code: "SMET-T-0253"
created_at: 2026-03-28T00:37:02.025207+00:00
updated_at: 2026-03-28T01:08:45.600117+00:00
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

# Integration Tests for Live Monitoring and Intervention

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Write end-to-end integration tests validating the complete monitoring and intervention flow: event ingestion, SSE streaming, approval handling, and guidance injection.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Test: Post events to ingestion endpoint, query them back via GET, verify ordering and content
- [ ] Test: Post an approval request event, verify it appears in pending approvals list
- [ ] Test: Respond to approval, verify pending_approvals record updated and response command queued
- [ ] Test: Respond to already-responded approval returns 400
- [ ] Test: Inject guidance, verify inject command queued and event persisted
- [ ] Test: Inject to terminal session returns 409
- [ ] Test: SSE stream delivers events in real-time (connect SSE, post event, verify received)
- [ ] Test: Event query with `since_sequence` pagination returns only newer events
- [ ] Test: Session ownership validation on all monitoring endpoints
- [ ] All tests pass alongside existing session, machine, and policy tests

## Implementation Notes

### Technical Approach
- Follow existing integration test pattern
- SSE test: use reqwest to connect to SSE endpoint, post an event in another task, read the SSE response
- May need `tokio::time::timeout` for SSE tests to avoid hanging
- Add helpers: `post_events`, `get_events`, `respond_to_approval`, `inject_guidance`, `get_pending_approvals`

### Dependencies
- All other tasks in this initiative (T-0246 through T-0252)

## Status Updates

*To be added during implementation*