---
id: integration-tests-for-session
level: task
title: "Integration Tests for Session Lifecycle"
short_code: "SMET-T-0232"
created_at: 2026-03-27T21:00:41.907299+00:00
updated_at: 2026-03-28T00:14:31.482583+00:00
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

# Integration Tests for Session Lifecycle

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Write end-to-end integration tests validating the complete session lifecycle: creation, state transitions, command routing, control actions, and error cases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Full lifecycle test: create → start command → running → completed
- [ ] Stop flow: create → running → stop → stopped
- [ ] Force stop: running → force-stop → stopped
- [ ] Pause/resume: running → pause → paused → resume → running
- [ ] Waiting for input: running → waiting_for_input → resume → running
- [ ] Invalid transitions: pause completed (409), stop stopped (409), resume running (409)
- [ ] Machine validation: session on pending machine (400), revoked machine (400), nonexistent (404)
- [ ] User scoping: user A's session not visible to user B
- [ ] Command queue: multiple commands queued, processed in order, each acknowledged
- [ ] All tests pass with `cargo test -p control-api`

## Implementation Notes

### Technical Approach
- Follow pattern from SMET-T-0204 (Machine Connectivity integration tests)
- In-process test server on random port with fresh SQLite
- Simulate runner via HTTP API calls
- Each test creates own machine/session for isolation
- Verify session state AND session_events for transitions

### Dependencies
- All other tasks (T-0224 through T-0231)

## Status Updates

*To be added during implementation*