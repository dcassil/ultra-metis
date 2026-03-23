---
id: live-monitoring-and-intervention
level: initiative
title: "Live Monitoring and Intervention"
short_code: "SMET-I-0041"
created_at: 2026-03-17T19:56:53.311678+00:00
updated_at: 2026-03-17T19:56:53.311678+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: live-monitoring-and-intervention
---

# Live Monitoring and Intervention Initiative

## Context

Once sessions are running (SMET-I-0040), users need to observe them in near real-time and intervene when needed. This covers both passive monitoring (live output stream, session detail, multi-session list) and active intervention (responding to approval requests, injecting guidance, toggling default behaviors). This is the primary day-to-day interaction surface for users managing AI sessions remotely.

The critical design challenge is surfacing AI approval requests as structured prompts rather than raw terminal text — this requires the Machine Runner to detect Claude Code's approval hooks and translate them into structured events that the Control Service can surface to the dashboard.

**Pre-requisites**: SMET-I-0038, SMET-I-0039, SMET-I-0040.

**Components touched**: All three — Control Dashboard (monitoring UI, approval response UI, intervention controls), Control Service (event fanout, approval routing), Machine Runner (output capture, prompt detection, guidance injection).

## Goals & Non-Goals

**Goals:**
- Near real-time session output stream delivered to the dashboard (low enough latency to feel live on mobile)
- Output events distinguish: informational progress, warnings, structured prompts, errors, final summaries
- Session detail page: current task, current state, recent output, pending prompts, basic actions in one view
- Latest important event shown first; condensed summary of session progress available
- Multi-session list: all active sessions across all machines with sort/filter (by machine, repo, state, urgency, task)
- Badges/indicators for sessions needing approval, input, or review
- Structured approval prompt detection: surface AI decision requests as explicit prompts, not raw terminal text
- Approval response UI: accept/reject/explicit choice with optional note, fast from mobile
- Guidance injection: send a message, side note, or interrupting instruction to a running session
- Injected messages appear in the session trace
- Default behavior toggles (auto-accept under safe conditions) bounded by policy

**Non-Goals:**
- Push notifications (SMET-I-0042)
- Full session history and replay (SMET-I-0043)
- Policy enforcement for what can be auto-accepted (SMET-I-0044)

## Detailed Design

### Event Model
All session activity is represented as a stream of typed events. Event types:
- `OutputLine` (text, category: info/warning/error/summary)
- `ApprovalRequest` (question, options, context)
- `ApprovalResponse` (choice, note, responder)
- `GuidanceInjected` (message, injection_type: normal/side-note/interrupt)
- `StateChanged` (old_state, new_state)
- `DefaultToggled` (toggle_name, value)

### Machine Runner — Output Capture and Prompt Detection
- Reads AI process stdout/stderr line by line
- Classifies each line: normal output, warning, error, or potential approval request
- Approval detection: look for Claude Code's `UserPromptSubmit` hook or structured output markers indicating a tool approval request
- Detected approvals emit `ApprovalRequest` events with the question text and available options
- All events POSTed to Control Service event ingestion endpoint

### Control Service — Event Fanout
- Ingests events from Machine Runner via persistent connection
- Persists events to session event log (for history in SMET-I-0043)
- Fans out to live subscribers: dashboard clients connected to that session stream
- SSE (Server-Sent Events) endpoint: `GET /sessions/{id}/events/stream` — dashboard subscribes here
- Pending approval requests tracked as session state: `waiting_for_input`

### Control Service — Intervention API
- `POST /sessions/{id}/respond` — respond to approval: `{choice, note}`
- `POST /sessions/{id}/inject` — inject guidance: `{message, injection_type}`
- `PUT /sessions/{id}/defaults/{toggle}` — set a default behavior toggle

### Machine Runner — Guidance Injection
- Receives inject command from Control Service
- For `normal` and `side-note`: writes message to AI process stdin (or hook mechanism)
- For `interrupt`: sends a signal or structured input that Claude Code treats as urgent context
- Records injection as an event back to Control Service

### Control Dashboard — Monitoring UI
- Session detail page: tabbed view — Live Output | Pending Prompts | Session Info
- Live Output tab: scrolling event stream, auto-scrolls to bottom, color-coded by category
- Pending Prompts tab: structured approval card (question + option buttons + optional note field)
- Session list: sortable table with state, machine, urgency badge, elapsed time
- Multi-session view badges: red dot for awaiting approval, yellow for stale

## Multi-Tenancy Notes

### SSE Stream Scoping
- SSE stream endpoint `GET /sessions/{id}/events/stream` validates `session.user_id = current_user` before subscribing — a user cannot subscribe to another user's session stream
- The event fanout in the Control Service only delivers events to subscribers who own the session

### Intervention API Scoping
- `POST /sessions/{id}/respond` and `POST /sessions/{id}/inject` validate session ownership before routing the command to the Machine Runner
- Injection is user-attributed: injected messages carry `injected_by: user_id` in the event record (for audit trail in SMET-I-0043)

### Multi-Session Dashboard View
- Session list query: `WHERE user_id = :current_user AND state IN (active states)` — always user-scoped
- Urgency badges, sort, and filter all operate within the current user's session set
- **Future admin view**: a separate endpoint `GET /admin/sessions` (role-gated) would return cross-user sessions; not exposed in MVP

## Alternatives Considered

- **WebSocket instead of SSE for output streaming**: WebSocket is bidirectional but adds complexity; SSE is sufficient for server-push output streaming; intervention (injection/response) uses regular REST calls; SSE chosen for simplicity
- **Parse raw terminal output for approvals (no hook integration)**: fragile; Claude Code's output format could change; rejected in favor of using Claude Code's hook system (PreToolUse/UserPromptSubmit hooks) for reliable structured approval events
- **Full terminal emulator in dashboard**: high complexity, overkill for mobile; simple event stream with categorization is more useful than raw ANSI terminal; rejected

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope (minor)**

Relevant ADR decision points:
- **#1 Rename**: References to "Cadre" become "Cadre" in event types, API documentation, and dashboard labels.
- **#3 SDD-style execution**: The event model should accommodate events from orchestrated multi-subagent sessions. When the execution uses SDD-style dispatch, the monitoring system will see events from multiple subagents within a single session. Event types should include subagent identity (which task's subagent produced this output). The two-stage review events (spec compliance + code quality) should surface as distinct event types so users can monitor review outcomes.
- **#7 SubagentStart hook**: Monitoring should be able to observe that the SubagentStart hook fired for each subagent, confirming Cadre context injection. This is an informational event, not a blocking concern.

No changes needed for: #2 (peer dependency is install-level), #4 (worktree delegation is orthogonal to monitoring), #5 (task claiming is orthogonal), #6 (architecture hooks are Phase 4).

## Implementation Plan

1. Define event types and event log schema
2. Implement Machine Runner output capture and event emission
3. Implement approval request detection using Claude Code hooks
4. Implement Control Service event ingestion and persistence
5. Implement SSE stream endpoint for live event delivery to dashboard
6. Implement intervention API (respond to approval, inject guidance)
7. Implement Machine Runner guidance injection handler
8. Build dashboard session detail page with live output stream
9. Build approval response UI (card with options + note field)
10. Build guidance injection UI (message input with injection type selector)
11. Build multi-session list with urgency badges
12. End-to-end test: session runs → approval detected → user responds from dashboard → session resumes