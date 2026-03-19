---
id: policy-and-safe-execution
level: initiative
title: "Policy and Safe Execution"
short_code: "SMET-I-0044"
created_at: 2026-03-17T19:56:56.710835+00:00
updated_at: 2026-03-17T19:56:56.710835+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: policy-and-safe-execution
---

# Policy and Safe Execution Initiative

## Context

Remote AI sessions introduce significant security risk: a compromised control plane could issue commands that cause real harm to local repos and machines. The policy and safety model is the defense layer. It defines what remote sessions can and cannot do at both the machine level and the repo level, enforces those boundaries locally (independent of the control service), logs all sensitive actions, and surfaces violations clearly.

The critical design principle is **defense in depth**: the Control Service enforces policy centrally AND the Machine Runner enforces policy locally. Even if a command passes the Control Service, the Machine Runner must independently validate it before executing. Remote actions should never bypass local safety gates.

**Pre-requisites**: SMET-I-0038, SMET-I-0039, SMET-I-0040. Policy should be established before connecting real AI sessions.

**Components touched**: Control Service (policy storage, central enforcement, violation logging), Machine Runner (local policy enforcement, independent validation), Control Dashboard (policy management UI, violation surface, session mode display).

## Goals & Non-Goals

**Goals:**
- Machine-level and repo-level policy model: define what remote sessions can do (allowed actions, blocked action categories)
- Central policy enforcement in Control Service: validate commands before routing
- Local policy enforcement in Machine Runner: validate commands independently before executing (defense in depth)
- Policy violations surfaced clearly to user with reason; never silently bypassed
- Session mode visibility (normal / restricted / elevated) visible in dashboard at all times
- Unsupported or unsafe remote actions rejected rather than attempted
- All sensitive actions logged: who initiated, what action, when, outcome
- Policy management UI in Control Dashboard: configure machine and repo policies
- Policy constrains what can be auto-approved (connects to default behavior toggles in SMET-I-0041)
- Audit trail for policy violations and overrides

**Non-Goals:**
- Complex rule expressions or programmable policy DSL (simple allow/deny model for MVP)
- Cross-machine policy inheritance or org-level policies (post-MVP)
- Integration with external compliance systems (post-MVP)

## Detailed Design

### Policy Model
Two scopes: **machine-level** (applies to all sessions on that machine) and **repo-level** (applies to sessions in that repo).

Policy fields:
- `allowed_action_categories`: list of allowed action types (e.g., `read_files`, `write_files`, `run_tests`, `run_builds`, `git_operations`, `install_packages`, `network_access`)
- `blocked_action_categories`: explicit deny list (takes precedence over allowed)
- `max_autonomy_level`: cap on the autonomy level a session can be started with
- `require_explicit_approval_for`: list of specific actions that always need user approval regardless of session autonomy
- `session_mode`: normal | restricted | elevated (visible badge in dashboard)

### Control Service — Policy Enforcement
- On session create: validate requested autonomy\_level against machine's `max_autonomy_level`
- On command route to runner: validate the command type against repo and machine policy
- Policy violations: command rejected with `PolicyViolationError(reason, policy_scope, blocked_action)` — never silently bypassed
- Violation events logged to session event log and to an audit log

### Machine Runner — Local Policy Enforcement
- Machine Runner holds a local copy of its machine policy (fetched and cached at startup, refreshed periodically)
- Before executing any command received from Control Service: re-validate against local policy
- If Control Service sends a command that violates local policy (e.g., due to stale policy or a compromised service): reject with local policy error, log, notify service
- This is the critical defense-in-depth layer

### Control Dashboard — Policy Management
- Machine detail page includes policy editor: toggle action category allows/denies, set max autonomy level
- Session detail shows `session_mode` badge and lists active policy restrictions
- Policy violation log: queryable list of blocked actions with session, reason, timestamp

## Multi-Tenancy Notes

### Policy Ownership and Scoping
- Machine-level policies are owned by the machine's `user_id` — only the owning user can edit their machine's policy
- Repo-level policies are similarly scoped through the machine they apply to
- Policy reads (for enforcement): the Machine Runner fetches its policy using its API token, which resolves to a specific `user_id` — it can only fetch its own machine's policy

### Roles and Policy Interaction
- The `roles` table defines what a user _is allowed to configure_ in policy (e.g., a `restricted` role might not be allowed to set `elevated` session mode)
- **MVP**: default role has no restrictions; all policy settings are configurable
- Role capability checks are scaffolded in the policy update endpoint but always pass in MVP

### Future Org/Team Policies
- Org-level or team-level policies (set by an admin, inherited by all machines in that org/team) are a natural extension: add `org_policy` and `team_policy` tables, evaluate them in the enforcement chain before machine/repo policy
- The enforcement chain is designed with this in mind: `org_policy → team_policy → machine_policy → repo_policy` (most specific wins or most restrictive wins — to be decided in design phase)
- **MVP**: only machine and repo level policies exist; org/team levels are empty pass-throughs

### Audit Log
- All policy violation logs carry `user_id` of the session owner and `machine_id` — correctly attributed for future multi-user audit queries

## Alternatives Considered

- **Policy as code (HCL/YAML config file in repo)**: more DevOps-friendly but harder to manage from a mobile dashboard; rejected for MVP in favor of dashboard-managed policy stored in Control Service
- **Single global policy instead of machine + repo scopes**: simpler but doesn't allow per-machine trust tiers (e.g., dev machine trusted, shared server restricted); rejected
- **Trust all commands from Control Service in Machine Runner**: single enforcement point is simpler but catastrophically unsafe if service is compromised; defense in depth is non-negotiable

## Implementation Plan

1. Define policy data model (machine policy, repo policy, action category enum)
2. Implement policy storage and CRUD API in Control Service
3. Implement policy validation in command routing layer (Control Service)
4. Implement policy violation logging and violation event emission
5. Implement local policy cache in Machine Runner (fetch on start, refresh on timer)
6. Implement local policy validation in Machine Runner command handler
7. Build policy management UI in dashboard (machine and repo policy editors)
8. Build policy violation log view in dashboard
9. Test: configure blocked action → start session → trigger that action → verify rejected both at service and runner
10. Test: compromise scenario simulation — service sends blocked command → runner rejects independently