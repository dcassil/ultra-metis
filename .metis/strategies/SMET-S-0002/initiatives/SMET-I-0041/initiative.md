---
id: live-monitoring-and-intervention
level: initiative
title: "Live Monitoring and Intervention"
short_code: "SMET-I-0041"
created_at: 2026-03-17T19:56:53.311678+00:00
updated_at: 2026-03-28T01:09:12.034314+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#feature-remote-management"
  - "#feature-ui"
  - "#category-interface-layers"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: live-monitoring-and-intervention
---

# Interaction Queue and Web UI Initiative

## Context

With the bridge detecting prompts and the server managing sessions (SMET-I-0040), the final MVP piece is the user-facing layer: the web UI where users see pending interactions, respond to them, view active sessions, and launch new ones. This is the primary day-to-day interaction surface.

The detailed implementation design is in `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`. This initiative covers the Preact + Tailwind web SPA, the server's UI WebSocket push, and the REST API endpoints that the web UI consumes. This is **MVP initiative 3 of 3**.

**Pre-requisites**: SMET-I-0039 (Bridge Connectivity), SMET-I-0040 (Session Lifecycle and Adapter Layer).

**Components touched**: Web UI (`web/` — Preact SPA with all views and components), Server (`server/` — REST API, UI WebSocket push, static file serving, interaction queue persistence).

## Goals & Non-Goals

**Goals:**
- **Interaction Queue** (main screen): vertical list of pending interaction cards, sorted newest first, with interaction-specific controls
- **Approval cards**: two large buttons (Allow / Deny) with tool name and args
- **Confirm cards**: Approve / Deny with expandable detail and optional comment field
- **Choice cards**: tappable option cards, one per choice
- **Freeform cards**: text input with send button
- **Notification cards**: info display, no response needed
- **Session Dashboard**: grid/list of all active sessions with project name, status indicator, time since last activity
- **Session Detail**: header with status/uptime, current pending interaction inline, interaction history for session, stop button
- **Session Launcher**: form with machine selector, project dropdown, optional initial prompt, launch button
- Badge count on browser tab / PWA icon for pending interaction count
- "All clear. N sessions working." when queue is empty
- Mobile-first responsive layout: thumb-reach controls, large tap targets (44px min), readable monospace (13px min on mobile)
- Bottom navigation bar: Sessions | Queue | Machines | Launch
- Server UI WebSocket (`/api/ws`) pushing real-time events: interaction.new, interaction.resolved, session.updated, session.ended
- Interaction queue persisted to SQLite (survives server restarts)
- PWA manifest + service worker for add-to-home-screen on phone
- Embedded in server binary via `rust-embed` for single-binary deployment

**Non-Goals:**
- Push notifications (SMET-I-0042, post-MVP)
- Session history and replay (SMET-I-0043, post-MVP)
- Live terminal output streaming in web UI (post-MVP — session detail shows status + interactions only)
- Guidance injection into running sessions (post-MVP)
- Default behavior toggles / auto-accept (SMET-I-0044, post-MVP)

## Detailed Design

See full spec: `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`

### Web UI Technology
- **Framework**: Preact with TypeScript (smaller bundle, ideal for mobile-first PWA)
- **Styling**: Tailwind CSS, mobile-first responsive
- **Build**: Vite, output embedded in server binary via `rust-embed`
- **Real-time**: WebSocket connection to `/api/ws` for instant updates
- **PWA**: Service worker + manifest for add-to-home-screen

### Web UI Views

**Interaction Queue (Queue.tsx)** — Main screen:
- Vertical list of cards, one per pending interaction, sorted newest first
- Each card shows: session badge (project name, status dot), interaction title, truncated context (expandable)
- Card controls vary by prompt_type: Allow/Deny buttons, option cards, text input + send
- Badge count in tab title / PWA icon
- Empty state: "All clear. N sessions working." with link to dashboard

**Session Dashboard (Dashboard.tsx)**:
- Grid/list of active sessions: project name, agent type badge, status indicator, time since last activity
- Tap to open session detail

**Session Detail (SessionDetail.tsx)**:
- Header: project name, status, uptime
- Current pending interaction (if any) with inline response controls
- Interaction history for this session
- Stop session button

**Session Launcher (Launch.tsx)**:
- Select machine (MVP: only one), select project (from bridge's available projects), optional initial prompt text area, launch button

### Web UI Components
- `InteractionCard.tsx` — base card
- `ApprovalCard.tsx` — Yes/No buttons
- `ConfirmCard.tsx` — Approve/Reject + comment
- `ChoiceCard.tsx` — option selection
- `FreeformCard.tsx` — text input
- `NotificationCard.tsx` — info display
- `SessionBadge.tsx` — status indicator

### Server — REST API (web UI endpoints)
```
GET  /api/sessions                    → List all active sessions
GET  /api/sessions/:id                → Session detail
POST /api/sessions                    → Launch new session (proxied to bridge)
DELETE /api/sessions/:id              → Cancel session (proxied to bridge)

GET  /api/interactions                → List pending interactions
GET  /api/interactions/:id            → Interaction detail
POST /api/interactions/:id/respond    → Submit response

GET  /api/machines                    → List connected bridges/machines
GET  /api/machines/:id/projects       → List available projects

WS   /api/ws                          → Real-time UI updates
```

### Server — UI WebSocket Events (`/api/ws`)
Separate from the `/bridge` WebSocket. Pushes UI-relevant events so the phone updates instantly:
```json
{ "event": "interaction.new", "data": { ... } }
{ "event": "interaction.resolved", "data": { "interaction_id": "..." } }
{ "event": "session.updated", "data": { ... } }
{ "event": "session.ended", "data": { ... } }
```

### Server — Interaction Queue Persistence
- Pending interactions stored in SQLite (survive server restarts)
- Interaction history (prompts + responses + timestamps) persisted
- Active sessions are in-memory only (rebuilt from bridge heartbeats on reconnect)

## Multi-Tenancy Notes

- `user_id` column on interactions table from day one — seeded to `user_id=1` in MVP
- All API queries include `WHERE user_id = :current_user` pattern
- MVP: no auth, single user, localhost-only — anyone on LAN can interact
- Future: API key auth or JWT; all downstream scoping already works

## Alternatives Considered

- **React instead of Preact**: larger ecosystem but bigger bundle. Preact's compat layer gives access to React libraries when needed; Preact chosen for PWA performance.
- **Next.js SSR**: adds server complexity; the SPA is embedded in the Rust server binary. Pure client-side rendering is simpler and sufficient for this use case. Rejected.
- **Full terminal emulator in dashboard**: high complexity, overkill for mobile; structured interaction cards are more useful than raw ANSI terminal. Rejected.
- **SSE instead of WebSocket for UI push**: simpler but unidirectional. WebSocket chosen because the UI also needs to send heartbeats and may need bidirectional communication for future features.

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope (minor)**

Relevant ADR decision points:
- **#1 Rename**: References to "Cadre" become "Cadre" in event types, API documentation, and dashboard labels.
- **#3 SDD-style execution**: The event model should accommodate events from orchestrated multi-subagent sessions. When the execution uses SDD-style dispatch, the monitoring system will see events from multiple subagents within a single session. Event types should include subagent identity (which task's subagent produced this output). The two-stage review events (spec compliance + code quality) should surface as distinct event types so users can monitor review outcomes.
- **#7 SubagentStart hook**: Monitoring should be able to observe that the SubagentStart hook fired for each subagent, confirming Cadre context injection. This is an informational event, not a blocking concern.

No changes needed for: #2 (peer dependency is install-level), #4 (worktree delegation is orthogonal to monitoring), #5 (task claiming is orthogonal), #6 (architecture hooks are Phase 4).

## Implementation Plan

1. Set up `web/` directory with Preact + TypeScript + Tailwind + Vite
2. Implement API client (`web/src/api/client.ts`) — REST calls to server
3. Implement WebSocket connection manager (`web/src/api/ws.ts`) — connect to `/api/ws`, handle reconnect
4. Implement state stores: sessions and interactions (Preact signals or zustand)
5. Build InteractionCard base component and all prompt-type variants (Approval, Confirm, Choice, Freeform, Notification)
6. Build Queue view — main screen with pending interaction list
7. Build Dashboard view — active sessions grid
8. Build SessionDetail view — status + pending interaction + interaction history + stop button
9. Build Launch view — machine/project selector + prompt textarea + launch button
10. Build bottom navigation bar and PWA manifest
11. Implement server `/api/ws` endpoint for UI push events
12. Implement server static file serving via `rust-embed` (embed built web assets in server binary)
13. Implement server REST API: interaction list, detail, respond endpoints
14. End-to-end test: bridge wraps Claude → prompt detected → appears in phone Queue view → tap respond → flows back to Claude → session resumes