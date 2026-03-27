---
id: integration-tests-for-machine
level: task
title: "Integration Tests for Machine Connectivity and Trust"
short_code: "SMET-T-0204"
created_at: 2026-03-27T16:18:46.139288+00:00
updated_at: 2026-03-27T21:07:51.079923+00:00
parent: SMET-I-0039
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0039
---

# Integration Tests for Machine Connectivity and Trust

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Write end-to-end integration tests that validate the complete machine connectivity and trust lifecycle: registration, pending state, approval, heartbeat, revocation, and the runner's response to each state transition. These tests exercise the Control Service API, Machine Runner client, and their interaction together.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Integration test: Full registration-to-approval flow — runner registers, appears as pending in `GET /machines`, user approves via `POST /machines/{id}/approve`, runner's next heartbeat succeeds, machine shows as trusted+online in `GET /machines`
- [ ] Integration test: Pending state blocking — runner registers, machine is pending, runner's heartbeat returns 403, runner remains in waiting state and does not process commands
- [ ] Integration test: Revocation flow — machine is trusted and online, user calls `POST /machines/{id}/revoke`, runner's next heartbeat returns 401, runner enters stopped state, machine shows as revoked in `GET /machines/{id}`
- [ ] Integration test: Heartbeat liveness — runner sends heartbeats, machine shows as online. Stop heartbeats, wait > 30 seconds (or mock time), machine status transitions to stale. Wait > 5 minutes (or mock time), status transitions to offline.
- [ ] Integration test: Repo discovery update — runner registers with repos A and B, sends heartbeat with repos A, B, and C (new repo cloned), `GET /machines/{id}` now shows all three repos. Runner sends heartbeat with only A and C (B deleted), detail shows A and C.
- [ ] Integration test: Invalid token — runner sends registration request with an invalid API token, receives 401, registration fails
- [ ] Integration test: User scoping — register two machines under different user_ids (if multi-tenancy seed supports it, or mock), verify `GET /machines` for user 1 does not return user 2's machines
- [ ] All integration tests pass in CI and can be run with a single command (e.g., `make test-integration` or `cargo test --test integration`)

## Test Cases

### Test Case 1: Registration to Approval Lifecycle
- **Test ID**: TC-001
- **Preconditions**: Control Service running with seeded database, valid machine token available
- **Steps**: 
  1. Runner sends `POST /machines/register` with name "test-machine", platform "linux/x86_64", repos ["repo-a"]
  2. Assert response status 201, response contains machine_id, status is "pending"
  3. Call `GET /machines` — assert "test-machine" appears with status "pending"
  4. Call `POST /machines/{id}/approve`
  5. Assert response status 200, machine status is now "trusted"
  6. Runner sends `POST /machines/{id}/heartbeat`
  7. Assert response status 200
  8. Call `GET /machines` — assert "test-machine" shows as "trusted", connectivity "online"
- **Expected Results**: Machine transitions through pending -> trusted -> online smoothly

### Test Case 2: Revocation Stops Runner
- **Test ID**: TC-002
- **Preconditions**: A trusted, online machine exists
- **Steps**: 
  1. Call `POST /machines/{id}/revoke`
  2. Assert response status 200
  3. Runner sends `POST /machines/{id}/heartbeat`
  4. Assert response status 401
  5. Call `GET /machines/{id}` — assert status is "revoked"
- **Expected Results**: Revoked machine gets 401 on heartbeat, shows as revoked in API

### Test Case 3: Heartbeat Connectivity Transitions
- **Test ID**: TC-003
- **Preconditions**: A trusted machine with recent heartbeat
- **Steps**:
  1. Verify machine connectivity is "online" (last heartbeat < 30s ago)
  2. Advance clock or wait past 30s without heartbeat
  3. Verify machine connectivity is "stale"
  4. Advance clock or wait past 5m without heartbeat
  5. Verify machine connectivity is "offline"
- **Expected Results**: Connectivity status degrades correctly over time

## Implementation Notes

### Technical Approach
- Integration tests should spin up the Control Service in-process (using the test server pattern — start the Axum/Actix app on a random port, run tests against it, tear down after)
- Use a test database (SQLite in-memory or a temporary Postgres instance) with migrations applied and seed data loaded
- For time-dependent tests (heartbeat staleness), either mock the clock or use a configurable "now" function that tests can override
- The Machine Runner client logic can be tested by calling the HTTP client functions directly against the test server — no need to start the full runner daemon process
- Group all integration tests in a `tests/integration/` directory or a dedicated `tests/machine_connectivity.rs` file

### Dependencies
- All other tasks in this initiative (SMET-T-0197 through SMET-T-0203) must be complete — integration tests exercise the full stack
- CI pipeline must support running integration tests (database setup, etc.)

### Risk Considerations
- Time-dependent tests are flaky if they rely on real wall-clock time — prefer clock mocking or generous time margins
- Database cleanup between tests is critical — each test should start with a clean state to avoid cross-test contamination
- Integration tests are slower than unit tests — keep the suite focused on critical paths, don't duplicate unit test coverage

## Status Updates

*To be added during implementation*