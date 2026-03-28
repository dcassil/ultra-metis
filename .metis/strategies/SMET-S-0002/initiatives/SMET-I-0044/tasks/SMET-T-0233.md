---
id: policy-data-model-and-database
level: task
title: "Policy Data Model and Database Schema"
short_code: "SMET-T-0233"
created_at: 2026-03-28T00:15:46.991101+00:00
updated_at: 2026-03-28T00:20:51.366955+00:00
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

# Policy Data Model and Database Schema

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Define the policy data model and extend the SQLite schema to support machine-level and repo-level policies, action categories, and session modes. This is the foundation for all policy enforcement.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ActionCategory` enum: `ReadFiles`, `WriteFiles`, `RunTests`, `RunBuilds`, `GitOperations`, `InstallPackages`, `NetworkAccess`, `WorktreeOperations`, `ShellExecution`
- [ ] `SessionMode` enum: `Normal`, `Restricted`, `Elevated`
- [ ] `MachinePolicy` struct: `id`, `machine_id`, `allowed_categories` (JSON array), `blocked_categories` (JSON array), `max_autonomy_level`, `session_mode`, `require_approval_for` (JSON array), `created_at`, `updated_at`
- [ ] `RepoPolicy` struct: `id`, `machine_id`, `repo_path`, `allowed_categories`, `blocked_categories`, `max_autonomy_level`, `require_approval_for`, `created_at`, `updated_at`
- [ ] `machine_policies` table in `db.rs` with all columns, FK to machines
- [ ] `repo_policies` table in `db.rs` with all columns, FK to machines
- [ ] Default machine policy seeded on machine approval: all categories allowed, max_autonomy=normal, session_mode=normal
- [ ] `is_action_allowed(action, machine_policy, repo_policy) -> Result<(), PolicyViolation>` — blocked takes precedence over allowed, repo policy narrows machine policy
- [ ] All types implement `Serialize`/`Deserialize` with roundtrip unit tests
- [ ] Existing tests pass

## Implementation Notes

### Technical Approach
- Follow existing `MachineStatus`/`TrustTier` enum patterns in `models.rs`
- Action categories stored as JSON arrays in SQLite TEXT columns
- Policy evaluation: blocked_categories checked first (deny wins), then allowed_categories checked, repo policy further restricts machine policy
- `PolicyViolation` struct: `reason: String`, `policy_scope: String` (machine/repo), `blocked_action: String`

### Dependencies
- Existing `db.rs` and `models.rs` in control-api

## Status Updates

*To be added during implementation*