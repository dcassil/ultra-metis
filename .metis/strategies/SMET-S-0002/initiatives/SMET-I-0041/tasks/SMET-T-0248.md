---
id: control-service-event-ingestion
level: task
title: "Control Service Event Ingestion and SSE Stream"
short_code: "SMET-T-0248"
created_at: 2026-03-28T00:36:56.904619+00:00
updated_at: 2026-03-28T00:53:55.276002+00:00
parent: SMET-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0041
---

# Control Service Event Ingestion and SSE Stream

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Implement the Control Service's event ingestion endpoint and SSE (Server-Sent Events) stream for live event delivery to dashboard clients.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `POST /api/sessions/{id}/events` — Batch event ingestion endpoint. Accepts array of events from runner. Persists to `session_output_events` table. MachineTokenAuth.
- [ ] `GET /api/sessions/{id}/events` — Query persisted events with `since_sequence` param for pagination. Returns events after the given sequence number. DashboardAuth.
- [ ] `GET /api/sessions/{id}/events/stream` — SSE endpoint. Dashboard connects, receives live events as they arrive. DashboardAuth. Validates session ownership.
- [ ] SSE stream sends events as JSON-encoded `data:` lines with `event:` type field
- [ ] In-memory broadcast channel (`tokio::sync::broadcast`) for live event fanout — ingested events pushed to all active SSE subscribers for that session
- [ ] SSE sends a heartbeat comment (`: keepalive`) every 15 seconds to prevent connection timeout
- [ ] `GET /api/sessions/{id}/approvals` — List pending approval requests (status=pending). DashboardAuth.
- [ ] Approval requests also persisted to `pending_approvals` table when ingested
- [ ] Unit tests for event persistence and query
- [ ] Existing tests pass

## Implementation Notes

### Technical Approach
- SSE in Axum: use `axum::response::Sse` with `tokio_stream` to convert broadcast receiver to an SSE stream
- Broadcast channel stored in `AppState`: `HashMap<String, broadcast::Sender<SessionEvent>>` wrapped in `Arc<Mutex<>>`
- On event ingestion: persist to DB, then send to broadcast channel if subscribers exist
- SSE endpoint: look up or create broadcast channel for session, subscribe, and stream
- `axum-extra` may be needed for SSE support — check if `axum` 0.8 has it built-in

### Dependencies
- SMET-T-0246 (Event Model and Schema)
- `tokio-stream` crate for SSE streaming

## Status Updates

*To be added during implementation*