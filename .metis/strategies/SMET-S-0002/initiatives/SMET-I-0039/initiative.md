---
id: machine-connectivity-and-trust
level: initiative
title: "Machine Connectivity and Trust"
short_code: "SMET-I-0039"
created_at: 2026-03-17T19:56:51.580311+00:00
updated_at: 2026-03-17T19:56:51.580311+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: machine-connectivity-and-trust
---

# Machine Connectivity and Trust Initiative

## Context

This is the foundational initiative for the Remote AI Operations Layer (SMET-S-0002). Before any session can be started remotely, the system needs a reliable model for registering, identifying, trusting, and revoking local machines. A machine is any computer where the Machine Runner daemon is installed and where AI sessions will execute. Without a solid machine registry and trust model, the rest of the system cannot safely route commands or receive events.

**Pre-requisite**: SMET-I-0038 (Monorepo Restructure) must be complete — this initiative creates code in `apps/machine-runner/` and `apps/control-api/`.

**Components touched**: Control Service (machine registry API), Machine Runner (registration + heartbeat client), Control Dashboard (machine list + detail views).

## Goals & Non-Goals

**Goals:**
- Machine registration API in Control Service (name, identity, status, metadata)
- Machine Runner daemon can register itself with the Control Service on startup
- Heartbeat mechanism so the Control Service knows machine online/offline/stale status
- Repo/working directory discovery: Machine Runner advertises what repos are available
- Trust model: explicit approval required before a machine can receive commands
- Machine revocation: user can disable a machine at any time from the dashboard
- Machine-level permissions and trust tiers (some machines more trusted than others)
- Machine list and machine detail views in the Control Dashboard

**Non-Goals:**
- Session management (covered in SMET-I-0040)
- Policy enforcement for what sessions can do on a machine (SMET-I-0044)
- Advanced machine health metrics or telemetry (post-MVP)

## Detailed Design

### Control Service — Machine Registry API
- `POST /machines/register` — register a new machine (name, platform, capabilities)
- `POST /machines/{id}/heartbeat` — periodic liveness signal from runner
- `GET /machines` — list all machines with status, last heartbeat, active session count
- `GET /machines/{id}` — machine detail: metadata, repos, trust level, active sessions
- `POST /machines/{id}/approve` — approve a pending machine for command receipt
- `POST /machines/{id}/revoke` — revoke a machine immediately
- Machine status computed from heartbeat recency: online (< 30s), stale (30s–5m), offline (> 5m)

### Machine Runner — Registration and Heartbeat
- On startup: POST to control service with machine name (from local config) and discovered repo list
- Periodic heartbeat (every 15–30s) keeps machine marked online
- Repo discovery: scan configured directories, expose repo name + path pairs
- If registration is rejected/pending, runner enters waiting state (no command processing)

### Control Dashboard — Machine Views
- Machine list: name, status badge (online/stale/offline/unhealthy), active sessions, last heartbeat
- Machine detail: expandable metadata, available repos, trust tier, revoke button
- Pending machines queue: approve or reject new registrations
- Trust tier badge shown in machine list and session start flow

### Trust Model
- New machines start in `pending` state — no commands accepted
- User approves explicitly from dashboard → machine moves to `trusted`
- Trusted machines can receive session commands for their allowed repos
- Revoked machines immediately stop receiving commands; runner gets 401 on next request
- Trust tiers (trusted, restricted): restricted machines need stricter approval on sessions

## Multi-Tenancy Notes

### Schema Changes
- `machines` table gets `user_id`, `team_id`, `org_id` foreign keys (all non-null; seeded to defaults in MVP)
- `repos` exposed by a machine are also scoped through the machine's `user_id`
- `roles` table: `(id, name, permissions_json)` — seed with `{id:1, name:"default", permissions_json:"{}"}`
- `user_roles` table: `(user_id, role_id)` — seed MVP user with role `default`
- Org/Team/User tables seeded at first startup with a single default record each

### API Scoping
- Machine registration: runner includes a machine API token in the request; Control Service resolves token → `user_id` → scopes the machine to that user
- **MVP**: runner uses a hardcoded static token; middleware maps it to `user_id=1`
- `GET /machines` always adds `WHERE user_id = :current_user` — never returns all machines globally
- Machine approval, revocation, and heartbeat endpoints validate the machine belongs to `current_user` before proceeding

### Auth Middleware (MVP)
- Middleware exists in the request pipeline and sets `request.user_id = 1` unconditionally
- Same middleware will be replaced with JWT verification when auth is added — no other code changes needed

### Dashboard
- Machine list and detail views render data from the user-scoped API — no changes needed when real auth lands
- No "all machines" admin view in MVP; admin cross-user view is a future feature flag

## Alternatives Considered

- **Agent-based polling (runner polls for commands)**: simpler NAT traversal but higher latency for command delivery; rejected in favor of persistent outbound connection (WebSocket or SSE) from runner to service
- **OAuth device flow for machine registration**: more standard but adds complexity for a local daemon; rejected in favor of explicit dashboard-based approval which fits the trust model better
- **mDNS/local discovery instead of central registry**: works only on local network, breaks the remote control model; rejected

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope**

Relevant ADR decision points:
- **#1 Rename**: All references to "Cadre" in this initiative become "Cadre." The Machine Runner daemon runs in a Cadre-managed project, not an cadre project. API paths, config keys, and documentation references must use the Cadre namespace.
- **#3 SDD-style execution**: The Machine Runner must support the new execution model where a session may spawn multiple fresh subagents per task (not just a single long-running process). The process supervisor in the runner needs to handle orchestrated multi-subagent sessions, not just a single AI process. Session start commands may include subagent dispatch configuration.
- **#7 SubagentStart hook**: The Machine Runner must ensure the SubagentStart hook is active in the execution environment so that any subagents spawned during a remote session receive Cadre project context. The runner's environment setup should verify hook availability.

No changes needed for: #2 (superpowers dependency is orchestration-level, not runner-level), #4 (worktree delegation is execution-level), #5 (task claiming is orthogonal to machine connectivity), #6 (architecture hooks don't affect machine registration).

## Implementation Plan

1. Define machine data model (id, name, platform, status, trust\_tier, repos, last\_heartbeat)
2. Implement Control Service machine registry API (register, heartbeat, list, detail, approve, revoke)
3. Implement Machine Runner registration client and heartbeat loop
4. Implement repo discovery in Machine Runner (configurable directories)
5. Build Control Dashboard machine list and machine detail views
6. Build pending machine approval flow in dashboard
7. Integration test: runner registers → dashboard shows pending → user approves → runner accepted
8. Revocation test: user revokes → runner gets 401 → dashboard shows offline