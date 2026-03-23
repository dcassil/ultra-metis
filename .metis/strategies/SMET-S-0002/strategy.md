---
id: cadre-remote-ai-operations
level: strategy
title: "Cadre Remote AI Operations Layer"
short_code: "SMET-S-0002"
created_at: 2026-03-17T19:51:13.777599+00:00
updated_at: 2026-03-17T19:51:13.777599+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#strategy"
  - "#phase/shaping"


exit_criteria_met: false
risk_level: high
stakeholders: []
strategy_id: cadre-remote-ai-operations
initiative_id: NULL
---

# Cadre Remote AI Operations Layer Strategy

*This template includes sections for various types of strategic documents. Delete sections that don't apply to your specific use case.*

## Problem Statement

AI coding sessions today are locked to the machine where the developer is sitting. There is no way to start a session remotely, monitor its progress from a phone, respond to approval requests without being at your desk, or manage multiple AI agents running in parallel across different machines and repos. When a session needs a decision, the user has no way to know without polling a terminal.

Cadre needs a remote AI operations layer: a system that lets users register local machines, launch AI sessions on those machines, monitor live progress from any device, respond to prompts and approvals from mobile, and capture durable session history, outputs, and artifacts — all connected back into Cadre tasks, notes, and architecture guidance.

## Components

This strategy covers three tightly-coupled components that together deliver the remote operations capability:

- **Control Dashboard** (`apps/control-web/`): The user-facing web app (Next.js). Shows machines, sessions, live output, approvals, notifications, history, and artifacts. Mobile-first design.
- **Control Service** (`apps/control-api/`): The hosted coordination layer. Authenticates users and machines, manages session lifecycle, routes commands, ingests events, persists state and audit trail, enforces policy, triggers notifications, and connects sessions to Cadre records.
- **Machine Runner** (`apps/machine-runner/`): The local execution daemon running near the code. Connects to the control service, advertises repos and capabilities, starts/stops local AI sessions, captures output and prompts, enforces local policy, and returns structured updates and artifacts.

## User Requirements

### A. Machine Onboarding and Connectivity
- Register local machines with explicit name, status, and identity
- See machine online/offline/stale/unhealthy state
- Expose which repos/working directories are available per machine
- Restrict repos available for remote work
- Require explicit registration and approval for machine access
- Revoke machines and configure machine-level trust tiers

### B. Session Lifecycle
- Start AI sessions against a selected machine and repo with task title, instructions, and optional context
- Choose session autonomy level (normal approvals, stricter approvals, more autonomous)
- Attach an Cadre task, story, or initiative to a new session
- Preload notes, architecture guidance, or constraints into session context
- See session state (starting, running, waiting, paused, completed, failed, stopped)
- Stop, pause, resume, and force-end sessions

### C. Live Monitoring and Intervention
- View near real-time session output with frequent updates, readable on mobile
- Distinguish informational progress, warnings, prompts, errors, and summaries
- Session detail page: current task, state, recent output, pending prompts, actions
- Multi-session oversight: list all active sessions across all machines with filtering

### D. Interactive Control and Approvals
- Surface structured approval/decision requests — not raw terminal text
- Respond with accept, reject, or explicit choice plus optional note
- Inject guidance into a running session (normal message, side note, interrupting instruction)
- Toggle default behaviors (auto-accept under safe conditions) bounded by policy

### E. Notifications and Remote Responsiveness
- Push notifications when sessions need input, complete, fail, or become stuck
- Configurable notification types; urgent items (approvals) routed distinctly
- Notifications navigate directly to the relevant session and action
- Central inbox/queue of sessions needing attention

### F. Session History, Audit, and Replay
- View past sessions with timeline, prompts, responses, interventions, and result summary
- Search and filter history by machine, repo, task title, state, and outcome
- Chronological event stream showing user actions alongside AI actions

### G. Artifacts and Outputs
- Final session summary: what was attempted, completed, and remains
- List of files, notes, or outputs produced
- Structured output capture: milestones as distinct events, not buried in terminal text
- Export or link session results into Cadre workflow records

### H. Policy and Safety
- Machine- and repo-level policy controls limiting what remote sessions can do
- Clear surface of policy violations and blocked actions
- Session mode visibility (normal/restricted/elevated)
- All sensitive actions logged with initiator and timestamp
- Remote actions flow through validated API calls; machines enforce local restrictions independently

### I. Cadre Integration
- Launch sessions from Cadre stories, tasks, or initiatives
- Session context includes project, repo area, and linked work item
- Completed session results flow back into the related work item
- Relevant notes fetched at session start; note helpfulness scored at session end
- Session proposes new notes from confirmed findings
- Sessions aware of relevant architecture guidance and repo rules

### J. Mobile-First Usability
- Key session actions easy to use on a phone
- Important details summarized without dense logs
- Responsive layouts for phone, tablet, and desktop
- Session controls visible while scrolling

### K. Operational Management
- Run multiple sessions across one or more machines
- Machine capacity visibility and over-commitment warnings
- Detect machine disconnection during a session; distinguish failure from connection loss
- Sessions and history survive transient dashboard or service interruptions
- Central management of machine-level and repo-level settings

## Success Metrics

- User can register a local machine and see it in the dashboard within minutes
- User can start an AI session from the dashboard and see live output within 30 seconds
- Approval requests surface as structured prompts, not raw terminal text
- User can respond to an approval request from mobile in under 3 taps
- Push notifications delivered within 30 seconds of session events
- Session history persists and is queryable after the session ends
- Session results link back to Cadre work items
- Policy violations are surfaced and logged, never silently bypassed
- Dashboard is fully usable on mobile

## Multi-Tenant Architecture

The system is designed multi-tenant from the start, even though MVP ships with a single default user and no login. The goal is to make adding real users, teams, and orgs a configuration/feature-flag change rather than a data model migration.

### Tenant Data Model

```
Org
 └── Team (one or more per Org)
      └── User (one or more per Team)
           └── Dashboard (one per User — their personal view)
                └── Machine (owned by User)
                     └── Session (owned by User)
```

Every resource that a user "owns" (machine, session, notification, device token, policy, work item linkage, note proposal) carries a `user_id` foreign key. All API queries are filtered by the authenticated user's `user_id` from request context. Users cannot see or act on another user's resources.

**Roles**: A `roles` table is created with at minimum a `default` role. The MVP user is assigned the `default` role. Role-based access control logic is scaffolded but not enforced in MVP — the enforcement gates are in place and readable but always pass for the `default` role.

### MVP Implementation (Single Default User)
- A single `Org`, `Team`, and `User` record is seeded at first startup (e.g., `org_id=1`, `team_id=1`, `user_id=1`)
- All resources created in MVP are owned by `user_id=1`
- Auth middleware exists but always returns the hardcoded default user — no login flow, no session tokens
- The Dashboard shows all data belonging to `user_id=1` (i.e., everything in MVP)
- No admin UI, no user management, no role enforcement in MVP

### Future Multi-User Path
- Swap auth middleware to a real identity provider (JWT, OAuth, etc.) — all downstream scoping already works
- Add user registration/invitation flow and admin UI (separate initiative)
- Roles enforcement: check role capabilities at the middleware/service layer (scaffolding already in place)
- Team-level shared resources (e.g., shared machines visible to all team members) can be added as a policy on top of the existing `user_id` scoping

### Key Design Rules
- **Always scope DB queries by `user_id`** — never return unfiltered result sets from user-facing endpoints
- **Resource ownership is immutable** — once a machine is registered to a user, it cannot be transferred without an explicit migration (avoids implicit cross-user access)
- **The `user_id` comes from request context, never from the request body** — clients cannot impersonate other users by supplying a different ID

## Solution Approach

Three-component architecture where the Machine Runner is a lightweight local daemon that connects outbound to the Control Service (avoiding firewall/NAT issues), the Control Service is a hosted stateless API with persistent storage for session state and events, and the Control Dashboard is a mobile-first Next.js app consuming the Control Service API.

All three components depend on SMET-I-0038 (monorepo restructure) which creates the `apps/` directory scaffold. The Control Service and Machine Runner share a common event and session model. Implementation is organized into 8 cross-cutting epics as initiatives:

1. **Machine Connectivity and Trust** — registration, heartbeat, trust tiers, revocation, repo exposure
2. **Remote Session Lifecycle** — session creation, state machine, termination, context loading
3. **Live Monitoring and Intervention** — real-time output, session detail, multi-session oversight, approval handling, guidance injection
4. **Notifications and Mobile Control** — push delivery, inbox, mobile-first UX, notification routing
5. **Session History, Audit, and Replay** — event timeline, search/filter, audit trail, chronological replay
6. **Policy and Safe Execution** — machine/repo policy model, violation surfacing, action logging, isolation guarantees
7. **Cadre Work and Notes Integration** — work item linkage, note fetch/score, architecture awareness, result handoff
8. **Operational Reliability and Multi-Session Management** — concurrency, capacity visibility, disconnection detection, resilience, administration

## Scope

**In Scope:**
- `apps/control-web/` — Next.js dashboard
- `apps/control-api/` — coordination API service
- `apps/machine-runner/` — local execution daemon
- All 8 epic initiatives and their user stories (A–K above)
- Shared data models: machine, repo, session, prompt/approval, event/timeline, artifact, policy, notification, task linkage, note feedback
- Cadre integration (work item linkage, note fetch/score, architecture guidance awareness)
- Mobile-first responsive design for dashboard

**Out of Scope:**
- Cadre core engine capabilities — see SMET-S-0001
- Native mobile apps (web-responsive is sufficient for MVP)
- Self-hosted Control Service infrastructure (cloud-hosted only for MVP)
- Advanced analytics or ML on session data
- Cross-organization session sharing
- **MVP out of scope**: login/authentication, user registration, admin UI, role enforcement, multi-user isolation testing (data model is in place; enforcement deferred)

## Risks & Unknowns

- **Real-time output delivery**: Streaming session output machine → service → dashboard with low latency across NAT/firewalls requires careful protocol design (SSE, WebSocket, or polling)
- **Machine Runner security**: Local daemon accepting remote commands is a high-security surface; local policy enforcement must be robust before accepting any remote commands
- **Approval prompt detection**: Reliably detecting when Claude Code needs a human decision requires structured hooks or output parsing; brittle approaches break the core interaction model
- **Session state consistency**: If machine runner disconnects mid-session, reconstructing accurate state requires explicit session checkpointing
- **Mobile notification delivery**: Requires push notification service (FCM/APNs) — infrastructure dependency not in the core Rust stack
- **Tech stack expansion**: Next.js and TypeScript/Node.js are new to a currently Rust-only codebase; CI and monorepo tooling must accommodate both
- **Tenant isolation correctness**: All DB queries must be user-scoped; a missed WHERE clause exposes one user's data to another. Requires disciplined query patterns and integration tests that verify cross-user isolation even in MVP (so the pattern is validated before real users are added)

## Implementation Dependencies

**Hard prerequisite**: SMET-I-0038 (Monorepo Restructure) must complete before any initiative under this strategy begins.

**Epic dependency order**:
1. Machine Connectivity and Trust — foundational; all others depend on machines existing
2. Remote Session Lifecycle — core session model required before monitoring, approvals, or history
3. Policy and Safe Execution — establish safety model before connecting real AI sessions
4. Live Monitoring and Intervention — builds on running sessions
5. Session History, Audit, and Replay — builds on session event model
6. Notifications and Mobile Control — builds on session events and state
7. Cadre Work and Notes Integration — builds on session model and core engine (SMET-S-0001)
8. Operational Reliability and Multi-Session Management — cross-cutting; can overlap with 3–7

## Change Log

### 2026-03-17 — Initial Strategy
- **Change**: Created strategy document for the Remote AI Operations Layer
- **Rationale**: New major product capability requiring its own strategy. Pre-req on SMET-I-0038 (monorepo restructure).
- **Impact**: 8 new initiatives will be created under this strategy covering Control Dashboard, Control Service, and Machine Runner.

### 2026-03-17 — Multi-Tenant Architecture Added
- **Change**: Added multi-tenant data model (Org → Team → User → Dashboard → Machine → Session) as a core architectural constraint. All resources are user-scoped from day one. Roles table scaffolded with default role. MVP ships with single seeded user and passthrough auth middleware.
- **Rationale**: Team has multiple developers each working on their own projects/machines. Future goal is each developer has their own dashboard with isolated view. Designing this in from the start avoids a painful migration when the second user is added.
- **Impact**: Every initiative must scope DB schemas and API queries by `user_id`. Auth middleware is a required component even in MVP (passthrough). All 8 initiatives updated with multi-tenancy notes.