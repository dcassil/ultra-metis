---
id: remote-session-lifecycle
level: initiative
title: "Remote Session Lifecycle"
short_code: "SMET-I-0040"
created_at: 2026-03-17T19:56:52.787698+00:00
updated_at: 2026-03-27T20:59:54.284237+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: remote-session-lifecycle
---

# Remote Session Lifecycle Initiative

## Context

With machines registered (SMET-I-0039), the next foundational piece is the session model: how sessions are created, started, monitored at a state level, controlled, and terminated. A session represents one AI work run on a specific machine and repo. The session state machine is the backbone that all other epics (monitoring, approvals, history, notifications) build on.

**Pre-requisites**: SMET-I-0038 (Monorepo Restructure), SMET-I-0039 (Machine Connectivity and Trust).

**Components touched**: Control Service (session CRUD, state machine, routing), Machine Runner (local process management, session execution), Control Dashboard (session start flow, session state display).

## Goals & Non-Goals

**Goals:**
- Session creation API: start a session against a machine/repo with title, instructions, context
- Session state machine: starting → running → waiting → paused → completed/failed/stopped
- Session autonomy level selection at creation time (normal, stricter, more autonomous)
- Attach an Cadre work item (task/story/initiative) to a new session
- Optional context preloading: notes, architecture guidance, constraints
- Machine Runner can start, supervise, and stop local AI processes
- Session control API: stop, pause, resume, force-end
- Session state visible in Control Dashboard (elapsed time, last activity, current phase)
- Users can identify which sessions are waiting for their input

**Non-Goals:**
- Live output streaming (SMET-I-0041)
- Approval request handling (SMET-I-0041)
- Session history and replay (SMET-I-0043)
- Work item result handoff (SMET-I-0045)

## Detailed Design

### Session State Machine
States: `starting → running → waiting_for_input → paused → completed | failed | stopped`
- `starting`: session record created, command routed to runner, runner launching AI process
- `running`: AI process active, producing output
- `waiting_for_input`: AI has surfaced a prompt/approval; session blocked on user response
- `paused`: user-requested hold; AI process suspended or in a wait state
- `completed`: AI process exited cleanly with a result
- `failed`: AI process exited with error or runner lost the process
- `stopped`: user force-ended the session

### Control Service — Session API
- `POST /sessions` — create session: machine\_id, repo, title, instructions, autonomy\_level, work\_item\_id (optional), context (optional)
- `GET /sessions/{id}` — session detail: state, elapsed, last\_activity, current\_phase
- `GET /sessions` — list sessions (filterable by machine, repo, state)
- `POST /sessions/{id}/stop` — request graceful stop
- `POST /sessions/{id}/force-stop` — force terminate
- `POST /sessions/{id}/pause` — request pause
- `POST /sessions/{id}/resume` — resume from paused or waiting

### Machine Runner — Session Execution
- Receives session start command with instructions and context
- Spawns AI process (Claude Code) with provided instructions in configured repo directory
- Supervises process: monitors stdout/stderr, captures exit code
- Reports state transitions back to Control Service as events
- On stop/force-stop commands: sends SIGTERM / SIGKILL to AI process
- On pause: suspends process (SIGSTOP); on resume: SIGCONT

### Control Dashboard — Session Start Flow
- Machine selector → repo selector → task title + instructions form → autonomy level picker → optional work item attachment → optional context fields → Start button
- Active session list shows: title, machine, repo, state badge, elapsed time, last activity indicator
- "Waiting for input" sessions highlighted prominently

## Multi-Tenancy Notes

### Schema Changes
- `sessions` table gets `user_id`, `team_id`, `org_id` foreign keys — always populated from request context, never from client body
- Session creation: `user_id` is stamped from `request.user_id` (the auth middleware result), not from the POST body
- `work_item_id` linkage (from SMET-I-0045) is implicitly user-scoped because sessions are user-scoped

### API Scoping
- `POST /sessions`: `user_id` set from context, not payload — client cannot create sessions on behalf of another user
- `GET /sessions`: always `WHERE user_id = :current_user` — no global session list in user-facing endpoints
- Session control actions (stop, pause, resume, force-stop): validate `session.user_id = current_user` before executing — a user cannot stop another user's session
- Future: team-level shared sessions (e.g., visible to all team members) would add a `team_id` filter as an OR condition; base scoping unchanged

### Machine Runner Auth
- Machine Runner's static API token maps to `user_id` in the Control Service
- **MVP**: one token, one user — the runner's sessions are automatically owned by that user
- **Future**: team members can register machines under a shared team token, or each developer has their own token

## Alternatives Considered

- **Serverless session model (no persistent process)**: sessions as queued jobs rather than long-running processes; cleaner for scaling but doesn't fit the interactive model where users inject guidance into running sessions; rejected
- **SSH tunnel for command routing**: familiar but requires open SSH port on remote machine; the outbound connection model from Machine Runner avoids this; rejected
- **Session state in runner only (no central state)**: runner is authoritative; simpler but breaks multi-device monitoring and recovery after disconnection; rejected in favor of central state in Control Service

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope**

Relevant ADR decision points:
- **#1 Rename**: All references to "Cadre" become "Cadre." Session creation API, work item linkage, and documentation use Cadre namespace.
- **#3 SDD-style execution**: This is the most impacted initiative. The session model must account for the fact that a session may internally use SDD-style fresh-subagent-per-task dispatch (via `/cadre-execute`), not just a single long-running AI process. The session state machine needs to accommodate orchestrated execution where the top-level process is a dispatcher spawning subagents. The Machine Runner's process supervisor must handle this pattern. Session state events should distinguish orchestrator-level state from individual subagent states.
- **#5 Simple task claiming**: When multiple remote sessions target the same repo, the simple file-based task claiming mechanism (`.cadre/claims/`) prevents duplicate work. The session creation flow should check for existing claims on the target work item.
- **#7 SubagentStart hook**: Sessions started remotely must have the SubagentStart hook active so all subagents within that session inherit Cadre context. The session start flow should verify hook availability as a prerequisite.

No changes needed for: #2 (peer dependency is install-time concern), #4 (worktree delegation handled by execution layer), #6 (architecture hooks are Phase 4).

## Implementation Plan

1. Define session data model and state machine enum
2. Implement Control Service session CRUD API and state transition logic
3. Implement command routing: Control Service → Machine Runner (via persistent connection)
4. Implement Machine Runner process supervisor (spawn, monitor, stop, force-stop, pause, resume)
5. Implement session state event reporting from runner to service
6. Build Control Dashboard session start flow (machine → repo → instructions → autonomy → start)
7. Build session list view with state badges and elapsed time
8. Build session control actions (stop, force-stop, pause, resume) in dashboard
9. Integration test: start → running → stop; start → running → waiting → resume → completed