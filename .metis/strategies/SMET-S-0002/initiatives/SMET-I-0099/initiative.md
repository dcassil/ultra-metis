---
id: machine-level-debug-log-pipeline
level: initiative
title: "Machine-Level Debug Log Pipeline: Runner to API to Dashboard"
short_code: "SMET-I-0099"
created_at: 2026-03-28T16:55:19.405979+00:00
updated_at: 2026-03-28T18:07:27.574751+00:00
parent: SMET-S-0002
blocked_by: [SMET-I-0098]
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: machine-level-debug-log-pipeline
---

# Machine-Level Debug Log Pipeline: Runner to API to Dashboard Initiative

## Context

**Blocked by**: [[SMET-I-0098]] — Installable Machine Runner with System Tray UI

With the runner becoming an installable Tauri desktop app (SMET-I-0098), users will interact with it as a tray application rather than a terminal process. The runner's internal debug output (registration, heartbeats, errors, policy fetches, session supervision) currently goes to stderr via `tracing_subscriber::fmt::init()` — but in a tray app, there is no terminal. Logs disappear into the void unless explicitly captured.

Session-level output already has a complete pipeline: the runner captures stdout/stderr from Claude sessions, batches events, sends them to the control API via `POST /api/sessions/{id}/events`, stores them in `session_output_events`, and streams them live via SSE to the dashboard's `LiveOutput` component.

This initiative extends that pattern to **machine-level debug logs** — the runner's own tracing output (not session output) — so operators can view machine health, connectivity issues, and debug information from the Control Dashboard without needing local access to the machine. The desktop app's settings window (from I-0098) controls the `log_level` setting that determines what gets forwarded.

## Goals & Non-Goals

**Goals:**
- Machine runner sends its own tracing/debug logs to the control API
- Control API stores and serves machine-level logs (separate from session events)
- Dashboard has a "Logs" tab on the machine detail page with live streaming and history
- Log level controlled by the `log_level` setting in the desktop app's Settings → Logging section (from SMET-I-0098)
- Runner also respects `RUST_LOG` env var for headless/CLI mode

**Non-Goals:**
- Local log viewer in the Tauri desktop app (the tray app is minimal — log viewing is the dashboard's job)
- API server's own request/error logs in the UI (separate initiative if needed)
- Replacing or modifying the existing session output pipeline
- Log aggregation across multiple machines into a single view
- Long-term log retention or archival policies

## Architecture

### Overview

Three-layer pipeline mirroring the existing session event pattern:

1. **Machine Runner** — Custom `tracing` layer that captures log events and forwards them to the control API via a new endpoint, with batching and backpressure
2. **Control API** — New `machine_logs` table, `POST /api/machines/{id}/logs` ingestion endpoint, `GET /api/machines/{id}/logs` query endpoint, `GET /api/machines/{id}/logs/stream` SSE endpoint
3. **Dashboard** — New "Logs" tab on machine detail page reusing the `LiveOutput` component pattern

### Data Flow

```
Runner tracing layer → batch buffer (500ms / 50 events)
  → POST /api/machines/{id}/logs
    → machine_logs table (SQLite)
    → broadcast channel → SSE stream
      → Dashboard LiveOutput component
```

## Detailed Design

### Machine Runner Changes (embedded in Tauri desktop app)
- The custom tracing layer lives in the `cadre-machine-runner` library crate, so it works for both the Tauri desktop app and headless CLI mode
- Integrate with the `RunnerHandle` API (from I-0098): the tracing layer reads `log_level` from the shared `Settings` and filters accordingly
- In Tauri mode: logs go to the API (primary) + a local log file at `~/.config/cadre/runner.log` (secondary, for crash debugging)
- In headless/CLI mode: logs go to the API + stderr (current behavior preserved)
- Add `EnvFilter::from_default_env()` for `RUST_LOG` support in headless mode
- Implement a custom `tracing_subscriber::Layer` that captures formatted log events
- Buffer events and flush to control API on interval or batch size threshold (same 500ms/50-event pattern as session events)
- Graceful degradation: if API is unreachable, logs still go to local file/stderr, buffer is dropped (not retried)

### Control API Changes
- New `machine_logs` table: `id, machine_id, timestamp, level, target, message, fields_json`
- `POST /api/machines/{id}/logs` — batch ingestion endpoint
- `GET /api/machines/{id}/logs?level=&since=&limit=` — historical query with filtering
- `GET /api/machines/{id}/logs/stream` — SSE endpoint with broadcast channel (same pattern as session events)

### Dashboard Changes
- New "Logs" tab on machine detail page (`/machines/:id`)
- Reuse `LiveOutput` component with log-level color coding (DEBUG=gray, INFO=blue, WARN=yellow, ERROR=red)
- Level filter dropdown to filter displayed logs
- Toggle between live stream and historical query

## Alternatives Considered

1. **Write logs to a file, serve via API** — Simpler but requires file system access and doesn't support live streaming. Rejected because it doesn't work for remote machines.
2. **Use the existing session events pipeline** — Machine logs aren't scoped to a session, so they don't fit the session event model. Would conflate two different concerns.
3. **External log aggregation (e.g., Loki/ELK)** — Overkill for the current single-machine setup. Can revisit when multi-machine is a real need.

## Implementation Plan

1. Add `machine_logs` table and ingestion/query/stream API endpoints in control-api
2. Implement custom tracing layer in cadre-machine-runner library crate with batched forwarding (works in both Tauri and headless mode)
3. Wire tracing layer into Tauri desktop app (local log file + API forwarding) and headless binary (stderr + API forwarding)
4. Add `RUST_LOG` / `EnvFilter` support for headless mode
5. Add "Logs" tab to machine detail page in Control Dashboard (reuse LiveOutput pattern)
6. Integration tests for the full pipeline: runner → API → SSE → dashboard