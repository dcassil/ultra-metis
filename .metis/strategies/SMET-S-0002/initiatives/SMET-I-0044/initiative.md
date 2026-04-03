---
id: policy-and-safe-execution
level: initiative
title: "Policy and Safe Execution"
short_code: "SMET-I-0044"
created_at: 2026-03-17T19:56:56.710835+00:00
updated_at: 2026-03-28T00:36:20.492635+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#feature-remote-management"
  - "#category-quality-governance"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: policy-and-safe-execution
---

# Policy and Safe Execution Initiative

**Status: Post-MVP** — builds on Shepherd MVP (SMET-I-0039, 0040, 0041). Should be implemented before scaling beyond single-user localhost.

## Context

The Shepherd MVP runs localhost-only with no auth — acceptable for single-user. Before scaling to multi-machine or multi-user, a policy and safety model must be in place. Remote AI sessions introduce security risk: the policy layer defines what remote sessions can and cannot do, enforces boundaries locally (defense in depth), logs sensitive actions, and surfaces violations.

The critical design principle is **defense in depth**: the server enforces policy centrally AND the bridge enforces policy locally. Even if a command passes the server, the bridge must independently validate it. The MVP's interaction queue already provides human-in-the-loop for every prompt; policy adds machine-level and repo-level guardrails on top.

**Pre-requisites**: SMET-I-0039, SMET-I-0040, SMET-I-0041 (Shepherd MVP complete).

**Components touched**: Server (`server/` — policy storage, central enforcement, violation logging), Bridge (`bridge/` — local policy enforcement, independent validation), Web UI (`web/` — policy management UI, violation surface, session mode display).

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

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope (minor)**

Relevant ADR decision points:
- **#1 Rename**: References to "Cadre" become "Cadre" in policy documentation and dashboard labels.
- **#2 Superpowers as peer dependency**: Policy must account for the fact that Cadre invokes superpowers skills during execution. Policy rules around allowed actions should consider superpowers skill invocations (e.g., `superpowers:using-git-worktrees` creates worktrees — policy should govern whether that is permitted on a given machine/repo).
- **#3 SDD-style execution**: Policy enforcement must work with the orchestrated execution model. When a session dispatches multiple subagents, each subagent's actions are independently subject to policy. The policy layer should enforce at the action level regardless of whether the action originates from the main session process or a dispatched subagent.
- **#4 Git worktree delegation**: Worktree creation via superpowers is a policy-relevant action. The policy model's action categories should include worktree operations so machines can allow or deny worktree creation.

No changes needed for: #5 (task claiming is orthogonal to policy), #6 (architecture hooks are Phase 4 and would add conformance gates, not change policy model), #7 (SubagentStart hook is context injection, not a policy concern).

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