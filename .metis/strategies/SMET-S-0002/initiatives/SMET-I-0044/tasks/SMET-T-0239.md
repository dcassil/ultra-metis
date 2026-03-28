---
id: integration-tests-for-policy
level: task
title: "Integration Tests for Policy Enforcement"
short_code: "SMET-T-0239"
created_at: 2026-03-28T00:15:52.750297+00:00
updated_at: 2026-03-28T00:35:37.421921+00:00
parent: SMET-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0044
---

# Integration Tests for Policy Enforcement

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Write end-to-end integration tests validating policy enforcement at both the Control Service and Machine Runner levels, including the defense-in-depth scenario.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Test: Configure machine policy to block `install_packages` → create session with autonomy=autonomous → verify session created but autonomy capped if policy max is lower
- [ ] Test: Set max_autonomy_level to `normal` → create session with `autonomous` → verify 400 rejection
- [ ] Test: Set repo policy to block `git_operations` → verify effective policy merges correctly
- [ ] Test: Policy violation creates a violation record and session event
- [ ] Test: Default policy created on machine approval
- [ ] Test: Update policy via PUT, verify GET returns updated values
- [ ] Test: Query violations with filters (machine_id, session_id, date range)
- [ ] Test: Runner fetches policy via GET endpoint with bearer token
- [ ] Test: Defense-in-depth — simulate runner receiving command that violates local policy (runner should reject independently)
- [ ] All tests pass alongside existing session and machine tests

## Implementation Notes

### Technical Approach
- Follow the existing integration test pattern in `tests/integration.rs`
- Reuse existing helpers (register_and_approve_machine, create_session, etc.)
- Add new helpers: `get_machine_policy`, `update_machine_policy`, `get_violations`
- Defense-in-depth test: set up a machine policy via API, then have runner fetch it and validate a command locally

### Dependencies
- All other tasks in this initiative (T-0233 through T-0238)

## Status Updates

*To be added during implementation*