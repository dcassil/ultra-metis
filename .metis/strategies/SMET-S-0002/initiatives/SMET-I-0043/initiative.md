---
id: session-history-audit-and-replay
level: initiative
title: "Session History, Audit, and Replay"
short_code: "SMET-I-0043"
created_at: 2026-03-17T19:56:55.575481+00:00
updated_at: 2026-03-17T19:56:55.575481+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: session-history-audit-and-replay
---

# Session History, Audit, and Replay Initiative

## Context

After sessions complete, users need to understand what happened: what was attempted, what decisions were made, what user interventions occurred, and what the outcome was. This requires durable session history with a chronological event stream, structured result records, and search/filter capabilities across past sessions.

This initiative is also the audit trail for the system — every approval, intervention, and outcome must be queryable so that AI work is explainable and reviewable after the fact. The session history is also the mechanism for linking execution back into Cadre work items.

**Pre-requisites**: SMET-I-0038, SMET-I-0039, SMET-I-0040, SMET-I-0041 (events generated during monitoring become the history).

**Components touched**: Control Service (event persistence, history storage, search API), Control Dashboard (session history views, timeline replay, search/filter UI).

## Goals & Non-Goals

**Goals:**
- Durable session history: all sessions retained with key events, prompts, responses, and result summary
- Session history list with search/filter: by machine, repo, task title, state, outcome
- Session detail replay: chronological event stream showing AI actions and user interventions together
- Final outcome record: status (success/partial/failure), summary, changed artifacts, next steps
- Filter for problematic sessions (failed, required many interventions, took too long)
- Reopen and review a completed or failed session's timeline
- User interventions and approvals visible alongside AI actions in the replay view
- Session results clearly distinguish partial success, full success, and failure
- Structured output capture: milestones appear as distinct events, not buried in raw terminal text
- Warnings, policy violations, and unresolved issues captured explicitly

**Non-Goals:**
- Real-time output streaming (SMET-I-0041 — history is the persisted record of what monitoring captured)
- Exporting session results to Cadre work items (SMET-I-0045)
- Cross-session analytics or trend analysis (post-MVP)

## Detailed Design

### Event Persistence (Control Service)
- All session events from SMET-I-0041 are persisted to the event log as they arrive
- Events stored with: session\_id, timestamp, event\_type, payload (JSON)
- Session outcome record written on terminal state (completed/failed/stopped): status, summary, artifacts, next\_steps
- Event log is append-only; no deletion (audit requirement)

### Session History API
- `GET /sessions?state=completed&machine_id=X&repo=Y&from=Z` — filter historical sessions
- `GET /sessions/{id}/events` — full chronological event log for a session
- `GET /sessions/{id}/outcome` — final outcome record
- Search: full-text search on session title, free-text filter across outcomes and summaries
- Filter helpers: `?outcome=failed`, `?required_interventions_gt=3`, `?duration_gt=3600`

### Control Dashboard — History Views
- Session history tab (separate from active sessions list): chronological list of past sessions
- Session replay view: timeline of events, color-coded by type (AI output, user action, state change, approval)
- User interventions shown inline with AI events, visually distinguished (e.g., left/right alignment)
- Outcome card at top of replay view: status badge, summary paragraph, artifacts list
- Search bar with filter chips: machine, repo, state, outcome, date range

## Multi-Tenancy Notes

### Query Scoping
- All history queries go through the session table, which is already user-scoped (`WHERE user_id = :current_user`)
- Event log queries join through sessions: `WHERE session.user_id = :current_user` — users cannot read another user's event log even if they know the session ID
- Session outcome records are accessible only through their parent session's user scope

### Search
- Full-text search on session history is bounded to the current user's sessions — no cross-user search results
- **Future admin search** (`GET /admin/sessions?query=...`) would be a separate role-gated endpoint with explicit cross-user scope

### Audit Trail
- User interventions in the event log carry `actor_user_id` — when real auth lands, this correctly attributes each action to the user who performed it
- **MVP**: `actor_user_id = 1` for all interventions; still correct structure for future attribution

## Alternatives Considered

- **Store only outcomes, not full event logs**: cheaper storage but loses the ability to replay and audit interventions; rejected — full event log is required for auditability
- **Separate audit database from session storage**: clean separation but operational complexity for MVP; rejected — single store with good schema is sufficient
- **Client-side replay from stored event stream**: dashboard replays events locally; simpler but requires streaming entire event log to client on load; rejected in favor of server-side pagination

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope**

Relevant ADR decision points:
- **#1 Rename**: All references to "Cadre" become "Cadre." The title references "Cadre work items" which becomes "Cadre work items."
- **#3 SDD-style execution**: Session history must capture the full SDD execution structure: which subagents were dispatched, what task each handled, and the two-stage review results (spec compliance + code quality). The replay view should show subagent boundaries so reviewers can see per-task execution rather than a single undifferentiated stream. Execution records from the orchestrator (dispatch order, model selection, review verdicts) are first-class audit data.
- **#6 Architecture hooks**: When architecture lifecycle hooks (SMET-I-0069) are wired into execution (ADR Phase 4), conformance check results should be captured in the session history as auditable events. This is a future dependency but should be noted in the event schema design now.

No changes needed for: #2 (peer dependency is install-level), #4 (worktree usage would appear in history naturally as git events), #5 (task claiming is orthogonal), #7 (SubagentStart hook firing can be an event but doesn't change history architecture).

## Implementation Plan

1. Confirm event log schema from SMET-I-0041 includes all necessary fields for history
2. Implement session outcome writer (triggered on terminal state transition)
3. Implement session history list API with filtering and search
4. Implement session event log API (paginated chronological events)
5. Build session history list view in dashboard with filter chips
6. Build session replay view: timeline with user/AI event distinction
7. Build outcome card component (status, summary, artifacts, next steps)
8. Test: run session → complete → verify history list shows it → open replay → see all events in order
9. Test search: find sessions by title keyword, filter by outcome=failed