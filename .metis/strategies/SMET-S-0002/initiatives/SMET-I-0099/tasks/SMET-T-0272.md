---
id: machine-logs-table-and-api
level: task
title: "Machine Logs Table and API Endpoints"
short_code: "SMET-T-0272"
created_at: 2026-03-28T17:49:56.423768+00:00
updated_at: 2026-03-28T17:57:41.782157+00:00
parent: SMET-I-0099
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0099
---

# Machine Logs Table and API Endpoints

## Parent Initiative

[[SMET-I-0099]] — Machine-Level Debug Log Pipeline

## Objective

Add the `machine_logs` database table and the three API endpoints (ingestion, query, SSE stream) in the Control Service. This mirrors the session event pattern but for machine-level runner logs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `machine_logs` table: `id`, `machine_id`, `timestamp`, `level` (debug/info/warn/error), `target` (Rust module path), `message`, `fields_json` (optional structured fields)
- [ ] `POST /api/machines/{id}/logs` — Batch ingestion. Accepts `{logs: [{level, target, message, fields, timestamp}]}`. MachineTokenAuth. Persists to table, broadcasts to SSE.
- [ ] `GET /api/machines/{id}/logs` — Query with filters: `level` (min level), `since` (timestamp), `target` (prefix match), `limit` (default 100), `offset`. DashboardAuth.
- [ ] `GET /api/machines/{id}/logs/stream` — SSE endpoint for live log streaming. DashboardAuth. Same broadcast pattern as session events.
- [ ] Machine log types in `models.rs`: `MachineLogEntry`, `IngestLogsRequest`, `QueryLogsParams`
- [ ] Broadcast channel per machine in `AppState` (separate from session event channels)
- [ ] SSE heartbeat every 15 seconds
- [ ] Unit tests for ingestion, query with filters, and level filtering
- [ ] Existing tests pass

## Implementation Notes

### Technical Approach
- Follow the exact same pattern as session event ingestion (T-0248): `POST` persists + broadcasts, `GET` queries, `GET /stream` subscribes via SSE
- `AppState` gets `log_channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>` alongside existing `event_channels`
- Level filtering: when `level=warn`, return warn+error only (hierarchy: debug < info < warn < error)
- Register routes under `machine_routes` in `lib.rs`

### Dependencies
- Existing control-api infrastructure

## Status Updates

*To be added during implementation*