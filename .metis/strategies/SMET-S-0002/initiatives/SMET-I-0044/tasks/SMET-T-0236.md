---
id: machine-runner-local-policy
level: task
title: "Machine Runner Local Policy Enforcement"
short_code: "SMET-T-0236"
created_at: 2026-03-28T00:15:49.809335+00:00
updated_at: 2026-03-28T00:31:06.947496+00:00
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

# Machine Runner Local Policy Enforcement

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Implement the defense-in-depth layer: the Machine Runner fetches its machine policy from the Control Service, caches it locally, and independently validates commands before execution — even if the Control Service already approved them.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `policy.rs` module in `apps/machine-runner/src/` with `LocalPolicyCache` struct
- [ ] Runner `client.rs` extended with `fetch_policy(machine_id) -> MachinePolicy` — calls `GET /api/machines/{id}/policy` with bearer token
- [ ] Policy fetched on runner startup after registration is approved
- [ ] Policy refreshed periodically (every 5 minutes or configurable)
- [ ] Before starting any session, runner validates command against local policy: checks autonomy level, checks action categories
- [ ] If local policy check fails, runner rejects the command, reports `failed` state to control service with metadata `{"reason": "local_policy_violation", "detail": "..."}`
- [ ] If control service is unreachable for policy refresh, runner uses cached policy (stale but safe)
- [ ] Policy types in runner match the API response format (serde deserialization)
- [ ] Unit tests for local policy validation logic
- [ ] Unit tests for cache behavior (fresh, stale, refresh)

## Implementation Notes

### Technical Approach
- `LocalPolicyCache` holds `Option<MachinePolicy>` and `last_fetched: Option<DateTime<Utc>>`
- On command receipt, if cache is empty or stale (> 5 min), attempt refresh before validation
- If refresh fails and cache exists, use cached policy (log warning). If no cache at all, reject command as a safety measure.
- Policy validation is the same `is_action_allowed` logic but implemented in the runner crate (can share types via a shared struct definition or just redefine matching types)
- Wire into `process_command()` in `runner.rs`: before dispatching to supervisor, check local policy

### Dependencies
- SMET-T-0234 (Policy CRUD API — provides the GET endpoint)
- Existing runner infrastructure

## Status Updates

*To be added during implementation*