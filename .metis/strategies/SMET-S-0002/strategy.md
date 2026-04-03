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

This strategy covers three tightly-coupled components in a **separate `shepherd/` repository** (Rust workspace + web SPA):

- **Web UI** (`web/`): Mobile-first Preact + Tailwind PWA. Shows interaction queue, session dashboard, session launcher. Embedded in the server binary via rust-embed for single-binary deployment.
- **Central Server** (`server/`): Axum-based Rust server. WebSocket hub for bridge connections, REST API for web UI, session registry, interaction queue, SQLite persistence. Serves the embedded web UI.
- **Agent Bridge** (`bridge/`): Local Rust daemon on the dev machine. Connects outbound to the server, discovers projects, manages AI sessions via PTY allocation, detects prompts (hybrid hooks + pattern matching), injects responses. Contains the adapter layer with the AgentAdapter trait.
- **Protocol** (`protocol/`): Shared Rust crate defining all message types, envelopes, and serialization. The contract between bridge and server.

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

### Chosen Implementation: Shepherd

The implementation design is codified in `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`. The system is called **Shepherd** and lives in a **separate repository** from ultra-metis.

**Why a separate repo**: The remote management layer doesn't depend on ultra-metis crate internals — it communicates with Claude Code (which happens to have Metis tools) through a generic adapter. Keeping it separate avoids blocking on monorepo tooling, mixing Rust workspace + Node/React build chains, and coupling release cycles. Ultra-Metis integration (SMET-I-0045) can happen later via API calls between the systems.

### Architecture

Three components connected by a plugin protocol:

1. **Central Server** (Axum, Rust) — WebSocket hub for bridge connections, REST API for web UI, session registry, interaction queue, SQLite persistence, embedded SPA serving
2. **Agent Bridge** (Rust daemon) — runs on the dev machine, connects outbound to server via WebSocket, manages AI sessions via PTY allocation, detects prompts via hybrid hooks+pattern-matching strategy
3. **Web UI** (Preact + Tailwind, mobile-first PWA) — interaction queue, session dashboard, session launcher, embedded in server binary via rust-embed

The bridge connects **outbound** to the server (works through firewalls/NAT). A **plugin protocol** defines all messages over WebSocket: session lifecycle (register/deregister/heartbeat/start/cancel/sync), interaction prompts (approval/confirm/choice/freeform/notification), and output streaming (post-MVP).

The key abstraction is the **AgentAdapter trait** — the bridge's adapter layer that knows how to manage a specific AI tool. The Claude Code adapter is the MVP implementation; future adapters (Cursor, Aider, direct Claude API) implement the same trait.

### Initiative Organization

Implementation is organized into 8 cross-cutting initiatives. The first 3 constitute the Shepherd MVP, the remaining 5 are post-MVP extensions:

**MVP (Shepherd v0.1)**:
1. **Bridge Connectivity and Handshake** (SMET-I-0039) — bridge daemon, WebSocket connection, hello/welcome handshake, project discovery, reconnection
2. **Session Lifecycle and Adapter Layer** (SMET-I-0040) — session start/stop/cancel, PTY management, AgentAdapter trait, Claude Code adapter, prompt detection, response injection
3. **Interaction Queue and Web UI** (SMET-I-0041) — interaction types, web UI views (queue, dashboard, launcher, session detail), REST API, UI WebSocket push

**Post-MVP**:
4. **Notifications and Mobile Control** (SMET-I-0042) — push notifications, notification inbox
5. **Session History, Audit, and Replay** (SMET-I-0043) — event persistence, history search, timeline replay
6. **Policy and Safe Execution** (SMET-I-0044) — machine/repo policy model, violation surfacing, defense-in-depth
7. **Ultra-Metis Work and Notes Integration** (SMET-I-0045) — work item linkage, note fetch/score, architecture awareness, result handoff
8. **Operational Reliability and Multi-Session Management** (SMET-I-0046) — concurrency, capacity visibility, disconnection detection, resilience

## Scope

**In Scope:**
- **Separate `shepherd/` repository** — Rust workspace with protocol, server, bridge crates + web SPA
- `protocol/` — shared message types, envelopes, serialization (the contract)
- `server/` — Axum server with WebSocket hub, REST API, session registry, interaction queue, SQLite, embedded SPA
- `bridge/` — daemon with WebSocket client, session manager, PTY management, adapter layer
- `web/` — Preact + Tailwind mobile-first PWA (interaction queue, dashboard, launcher, session detail)
- All 8 initiatives (3 MVP + 5 post-MVP)
- Plugin protocol: session lifecycle, interaction prompts, output streaming, connection management
- AgentAdapter trait and Claude Code adapter (MVP), extensible to other AI tools
- Mobile-first responsive design
- `user_id` column on all data tables from day one (cheap now, painful to retrofit)

**Out of Scope:**
- Ultra-Metis core engine capabilities — see SMET-S-0001
- Native mobile apps (PWA is sufficient)
- Advanced analytics or ML on session data
- Cross-organization session sharing
- Modifying Claude Code source code
- **MVP out of scope**: authentication, push notifications, multi-machine support, output streaming in web UI, non-Claude-Code adapters, session persistence across server restarts

## Risks & Unknowns

- **Real-time output delivery**: Streaming session output machine → service → dashboard with low latency across NAT/firewalls requires careful protocol design (SSE, WebSocket, or polling)
- **Machine Runner security**: Local daemon accepting remote commands is a high-security surface; local policy enforcement must be robust before accepting any remote commands
- **Approval prompt detection**: Reliably detecting when Claude Code needs a human decision requires structured hooks or output parsing; brittle approaches break the core interaction model
- **Session state consistency**: If machine runner disconnects mid-session, reconstructing accurate state requires explicit session checkpointing
- **Mobile notification delivery**: Requires push notification service (FCM/APNs) — infrastructure dependency not in the core Rust stack
- **Tech stack expansion**: Next.js and TypeScript/Node.js are new to a currently Rust-only codebase; CI and monorepo tooling must accommodate both
- **Tenant isolation correctness**: All DB queries must be user-scoped; a missed WHERE clause exposes one user's data to another. Requires disciplined query patterns and integration tests that verify cross-user isolation even in MVP (so the pattern is validated before real users are added)

## Implementation Dependencies

**No hard prerequisite on ultra-metis monorepo** — Shepherd is a separate repository. SMET-I-0038 (Monorepo Restructure) is already complete but is not a blocker since Shepherd lives outside ultra-metis.

**MVP initiative dependency order** (all three are tightly coupled and should be built together):
1. **SMET-I-0039 — Bridge Connectivity and Handshake** — foundational; the bridge daemon, WebSocket connection, and project discovery
2. **SMET-I-0040 — Session Lifecycle and Adapter Layer** — core session model, PTY management, Claude Code adapter, prompt detection; depends on bridge existing
3. **SMET-I-0041 — Interaction Queue and Web UI** — the user-facing layer; depends on sessions and interactions flowing through the system

**Post-MVP dependency order**:
4. Policy and Safe Execution (SMET-I-0044) — establish safety model before scaling
5. Session History, Audit, and Replay (SMET-I-0043) — builds on session event model from MVP
6. Notifications and Mobile Control (SMET-I-0042) — builds on session events and state
7. Ultra-Metis Work and Notes Integration (SMET-I-0045) — requires both Shepherd and ultra-metis core engine (SMET-S-0001)
8. Operational Reliability and Multi-Session Management (SMET-I-0046) — cross-cutting; can overlap with 4–7

## Change Log

### 2026-03-17 — Initial Strategy
- **Change**: Created strategy document for the Remote AI Operations Layer
- **Rationale**: New major product capability requiring its own strategy. Pre-req on SMET-I-0038 (monorepo restructure).
- **Impact**: 8 new initiatives will be created under this strategy covering Control Dashboard, Control Service, and Machine Runner.

### 2026-03-17 — Multi-Tenant Architecture Added
- **Change**: Added multi-tenant data model (Org → Team → User → Dashboard → Machine → Session) as a core architectural constraint. All resources are user-scoped from day one. Roles table scaffolded with default role. MVP ships with single seeded user and passthrough auth middleware.
- **Rationale**: Team has multiple developers each working on their own projects/machines. Future goal is each developer has their own dashboard with isolated view. Designing this in from the start avoids a painful migration when the second user is added.
- **Impact**: Every initiative must scope DB schemas and API queries by `user_id`. Auth middleware is a required component even in MVP (passthrough). All 8 initiatives updated with multi-tenancy notes.

### 2026-03-19 — Shepherd Design Adopted
- **Change**: Adopted the Shepherd design (`docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`) as the implementation approach. Key decisions: (1) separate `shepherd/` repository instead of `apps/` in ultra-metis monorepo; (2) Preact + Tailwind PWA instead of Next.js; (3) plugin protocol with typed messages as the contract; (4) AgentAdapter trait as the core extension point; (5) hybrid hooks + terminal pattern matching for Claude Code prompt detection; (6) `shepherd-bridge wrap` for local session management.
- **Rationale**: Shepherd addresses the hard engineering problems (PTY management, prompt detection, protocol design) concretely. Separate repo avoids blocking on monorepo tooling, mixing build chains, and coupling release cycles. Ultra-Metis integration can happen via API calls later.
- **Impact**: Components renamed (Machine Runner → Agent Bridge, Control Service → Central Server, Control Dashboard → Web UI). Initiatives 0039–0041 updated to align with Shepherd MVP design. Initiatives 0042–0046 marked as post-MVP. Multi-tenancy data model (`user_id` on all tables) carried forward from prior decision — cheap to include from day one. SMET-I-0038 monorepo restructure is no longer a blocker.