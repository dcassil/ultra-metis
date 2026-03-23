---
id: operational-reliability-and-multi
level: initiative
title: "Operational Reliability and Multi-Session Management"
short_code: "SMET-I-0046"
created_at: 2026-03-17T19:56:58.462183+00:00
updated_at: 2026-03-17T19:56:58.462183+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: operational-reliability-and-multi
---

# Operational Reliability and Multi-Session Management Initiative

## Context

As the system matures, users will run multiple sessions concurrently across multiple machines. This initiative covers the operational concerns: concurrency and capacity management, machine disconnection detection and recovery, session resilience across service interruptions, and administration tools for managing machine and repo settings centrally.

This is a cross-cutting initiative that builds on all the foundational epics and can be developed in parallel with epics 4–7 once the core session model is stable.

**Pre-requisites**: SMET-I-0038, SMET-I-0039, SMET-I-0040. Can overlap with SMET-I-0041 through SMET-I-0045.

**Components touched**: Control Service (concurrency tracking, disconnection detection, state recovery, administration API), Machine Runner (heartbeat, capacity reporting, graceful shutdown, reconnection logic), Control Dashboard (multi-session list views, capacity indicators, admin settings views).

## Goals & Non-Goals

**Goals:**
- Multiple concurrent sessions across one or more machines: system supports parallel execution
- Machine capacity visibility: which machines are busy, how many sessions active, available capacity
- Over-commitment prevention: warn or block when a machine would be unsafe to add more sessions
- Machine disconnection detection during a session: state marked uncertain, user notified
- Distinguish true session failure from connection loss (so recovery is targeted)
- Sessions and history survive transient dashboard or Control Service interruptions
- Machine Runner reconnection: if connection to Control Service drops, runner re-establishes and reconciles state
- Session checkpointing: enough state persisted that a reconnect can resume without losing history
- Central administration: configure default session behavior, machine-level settings, repo-level settings from one place
- System health visibility: event flow, active connections, error rates (basic operational dashboard)

**Non-Goals:**
- Automatic session migration between machines on disconnection (post-MVP)
- Horizontal scaling of the Control Service (single-instance for MVP)
- Full SRE-style observability and alerting stack (basic health visibility only for MVP)

## Detailed Design

### Concurrency and Capacity
- Machine model includes `max_concurrent_sessions` (configurable per machine, default 1 for safety)
- Control Service enforces: reject session start if machine is at capacity with clear error
- Dashboard warns when a machine has sessions ≥ (max - 1) to allow user to plan
- Machine detail shows: active session count, max, capacity bar

### Disconnection Detection and Recovery
- Control Service tracks last heartbeat per machine (from SMET-I-0039)
- If machine goes stale (heartbeat gap > 60s) during an active session: session state set to `connection_lost`
- User notified: "Machine disconnected — session state unknown"
- `connection_lost` is distinct from `failed`: user can wait for reconnect or force-stop
- When Machine Runner reconnects: reconcile state — runner reports current session status, Control Service updates accordingly
- If runner reports session still active on reconnect: state restored to `running`; if runner reports session ended: state updated to appropriate terminal state

### Session Checkpointing
- Machine Runner periodically (every 30–60s) POSTs a checkpoint to Control Service: current session progress snapshot
- Checkpoint stored with session; used during reconciliation on reconnect
- Checkpoint content: last event timestamp, current phase, any pending prompts

### Resilience — Dashboard and Service Interruptions
- Dashboard uses SSE with automatic reconnect (browser EventSource handles this natively)
- Session state is authoritative in Control Service DB; dashboard re-fetches on reconnect
- Control Service is stateless beyond the database — can be restarted without losing session state
- Machine Runner buffers events locally during Control Service downtime (in-memory queue); flushes on reconnect

### Administration
- `GET /admin/settings` — default session behavior settings (default autonomy level, default notification preferences)
- `PUT /admin/settings` — update defaults
- Machine list management page: bulk approve/revoke, edit trust tiers, edit max concurrent sessions
- System health panel: active connections count, event throughput, last event timestamps per machine

## Multi-Tenancy Notes

### Capacity and Concurrency Per User
- `max_concurrent_sessions` is a per-machine setting, and machines are user-scoped — capacity limits apply per user's machines, not globally
- Capacity visibility in the dashboard shows only the current user's machines and their session counts
- **Future**: org-level capacity limits (e.g., max total concurrent sessions across a team) can be added as a team policy; the per-machine enforcement remains unchanged

### Administration Scoping
- `GET /admin/settings` in MVP returns global defaults (no user scoping needed since there's one user)
- **Future**: settings are per-user (notification defaults, session defaults) with org/team defaults that cascade down — the settings table gets `user_id`/`team_id`/`org_id` nullable columns; resolution order: user → team → org → global default
- Admin API endpoints (cross-user machine list, cross-user session list, user management) are gated by the `admin` role — scaffolded but not exposed in MVP

### System Health Panel
- **MVP**: health panel shows aggregate metrics across all sessions/machines (only one user anyway)
- **Future**: health panel for a regular user shows only their resources; admin health panel shows all — same query, different `user_id` filter (or no filter for admin)

### Seed Data
- First-startup seed creates: `orgs(id=1, name="default")`, `teams(id=1, org_id=1, name="default")`, `users(id=1, team_id=1, email="admin@localhost", name="Admin")`, `roles(id=1, name="default")`, `user_roles(user_id=1, role_id=1)`
- Seed is idempotent (skip if records exist) so it's safe to run on every startup

## Alternatives Considered

- **Machine Runner persists session to disk for recovery**: stronger durability but complex; in-memory buffer with Control Service as source of truth is sufficient for MVP; disk persistence deferred
- **Automatic session handoff to another machine on disconnection**: desirable but complex (requires identical repo state on target machine); rejected for MVP
- **Multiple Control Service instances (horizontal scale)**: requires distributed state; single instance with good uptime is sufficient for MVP; deferred

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope**

Relevant ADR decision points:
- **#1 Rename**: References to "Cadre" become "Cadre" in administration settings and dashboard labels.
- **#3 SDD-style execution**: Multi-session management must account for the fact that a single "session" using SDD-style execution may internally spawn multiple subagents, each consuming resources. The capacity model (`max_concurrent_sessions`) should consider whether it counts top-level sessions or total active subagents. Recommendation: count top-level sessions for simplicity in MVP, but track subagent count as metadata for capacity planning.
- **#4 Git worktree delegation**: When parallel sessions on the same machine target the same repo, worktree isolation (delegated to superpowers) prevents conflicts. The operational reliability layer should be aware that worktrees may exist and handle cleanup on session failure or machine disconnection.
- **#5 Simple task claiming**: With multiple concurrent sessions, simple file-based task claiming (`.cadre/claims/`) is the mechanism that prevents two sessions from working on the same task. This initiative's concurrency management should integrate with the claiming mechanism — session start should check claims, and session cleanup should release claims.

No changes needed for: #2 (peer dependency is install-level), #6 (architecture hooks are Phase 4), #7 (SubagentStart hook is per-session setup, not an operational concern).

## Implementation Plan

1. Add `max_concurrent_sessions` to machine model; implement capacity check in session creation
2. Build capacity indicator in dashboard machine detail and session start flow
3. Implement `connection_lost` state transition when heartbeat gap detected during active session
4. Implement runner reconnection and state reconciliation in Machine Runner and Control Service
5. Implement session checkpointing (runner → service every 30s)
6. Implement event buffer in Machine Runner for Control Service downtime
7. Test resilience: kill Control Service mid-session → restart → verify session state recovered
8. Test disconnection: kill Machine Runner mid-session → wait → reconnect → verify state reconciled
9. Build system administration settings page in dashboard
10. Build system health panel (active connections, event throughput)