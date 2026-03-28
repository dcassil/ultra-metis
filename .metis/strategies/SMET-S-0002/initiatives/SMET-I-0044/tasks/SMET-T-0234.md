---
id: policy-crud-api-and-central
level: task
title: "Policy CRUD API and Central Enforcement"
short_code: "SMET-T-0234"
created_at: 2026-03-28T00:15:47.988707+00:00
updated_at: 2026-03-28T00:25:49.652933+00:00
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

# Policy CRUD API and Central Enforcement

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Implement the policy CRUD API endpoints and integrate policy enforcement into the session creation and command routing flows.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `GET /api/machines/{id}/policy` — Returns the machine policy. DashboardAuth.
- [ ] `PUT /api/machines/{id}/policy` — Update machine policy (allowed/blocked categories, max_autonomy, session_mode, require_approval_for). DashboardAuth. Validates machine belongs to user.
- [ ] `GET /api/machines/{id}/repos/{repo_path}/policy` — Returns repo-specific policy (URL-encoded repo path). DashboardAuth.
- [ ] `PUT /api/machines/{id}/repos/{repo_path}/policy` — Update repo policy. DashboardAuth.
- [ ] `GET /api/machines/{id}/policy/effective?repo_path=X` — Returns the effective (merged) policy for a specific repo on a machine. Merges machine + repo policies with blocked taking precedence.
- [ ] Session creation enforces `max_autonomy_level`: if requested level exceeds machine policy cap, returns 400 with descriptive error
- [ ] Default policy created automatically when machine is approved (all categories allowed, max_autonomy=autonomous, session_mode=normal)
- [ ] Unit tests for CRUD operations and policy enforcement on session creation
- [ ] Existing tests pass

## Implementation Notes

### Technical Approach
- Policy CRUD follows same patterns as machine endpoints
- Effective policy merge: start with machine policy, then narrow with repo policy. Blocked categories from either scope are blocked. Allowed categories must be in both scopes.
- Session creation adds a policy check before inserting: load effective policy for (machine, repo), validate autonomy level
- Repo path in URL needs URL encoding/decoding since paths contain `/`

### Dependencies
- SMET-T-0233 (Policy Data Model)

## Status Updates

*To be added during implementation*