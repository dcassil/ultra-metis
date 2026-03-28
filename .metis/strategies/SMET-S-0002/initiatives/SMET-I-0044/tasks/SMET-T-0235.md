---
id: policy-violation-logging-and-audit
level: task
title: "Policy Violation Logging and Audit Trail"
short_code: "SMET-T-0235"
created_at: 2026-03-28T00:15:48.874370+00:00
updated_at: 2026-03-28T00:31:06.088493+00:00
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

# Policy Violation Logging and Audit Trail

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Implement policy violation logging, an audit trail table, and a query API for violations. Every blocked action must be recorded with full context for security audit.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `policy_violations` table: `id`, `session_id`, `machine_id`, `user_id`, `action`, `policy_scope` (machine/repo), `reason`, `repo_path`, `timestamp`
- [ ] When a policy check fails (in session creation or command routing), a violation record is inserted automatically
- [ ] `GET /api/policy-violations` — List violations for current user, filterable by `machine_id`, `session_id`, `date_from`, `date_to`. DashboardAuth. Paginated.
- [ ] `GET /api/sessions/{id}/violations` — List violations for a specific session. DashboardAuth.
- [ ] Violation also inserted as a `session_event` with type `policy_violation` so it appears in the session timeline
- [ ] Violations are never silently swallowed — every policy rejection must log
- [ ] Unit tests for violation insertion and query

## Implementation Notes

### Technical Approach
- Violation logging is a side effect of the `is_action_allowed` check — wrap it in a higher-level function that both checks and logs
- Violations query follows the same pattern as session list (filter + paginate)
- Session event integration: insert a session_event alongside the violation so the dashboard timeline shows it

### Dependencies
- SMET-T-0233 (Policy Data Model), SMET-T-0234 (Policy CRUD — enforcement triggers violations)

## Status Updates

*To be added during implementation*