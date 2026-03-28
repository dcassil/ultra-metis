---
id: custom-tracing-layer-with-batched
level: task
title: "Custom Tracing Layer with Batched API Forwarding"
short_code: "SMET-T-0273"
created_at: 2026-03-28T17:49:57.636463+00:00
updated_at: 2026-03-28T18:01:29.814238+00:00
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

# Custom Tracing Layer with Batched API Forwarding

## Parent Initiative

[[SMET-I-0099]] — Machine-Level Debug Log Pipeline

## Objective

Implement a custom `tracing_subscriber::Layer` in the machine-runner crate that captures log events and forwards them in batches to the Control Service API. This layer is the bridge between Rust's tracing ecosystem and the remote log viewer.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `log_forwarder.rs` module in `apps/machine-runner/src/`
- [ ] `LogForwardingLayer` struct implementing `tracing_subscriber::Layer<S>`
- [ ] Captures: level, target (module path), message, structured fields, timestamp
- [ ] Respects the `log_level` from `Settings` — filters events below the configured level
- [ ] Batches events: flushes every 500ms or when batch hits 50 events (whichever first)
- [ ] Uses `client.post_machine_logs(machine_id, logs)` to send batches to the API
- [ ] `ControlClient` extended with `post_machine_logs(machine_id, logs)` — POST to `/api/machines/{id}/logs`
- [ ] Graceful degradation: if API call fails, log to stderr and drop the batch (no retry, no infinite loop)
- [ ] Layer is `Send + Sync` so it works with tokio's multi-threaded runtime
- [ ] Unit tests for event capture and batch logic

## Implementation Notes

### Technical Approach
- Use `tracing_subscriber::Layer` trait with `on_event` callback
- Inside `on_event`: format the event into a `LogEntry` struct, push to a `mpsc::UnboundedSender`
- A background tokio task reads from the receiver, buffers, and flushes on interval/size threshold
- The layer itself must be non-blocking (just sends on channel)
- `LogForwardingLayer::new(client, machine_id, settings)` returns `(Self, JoinHandle<()>)` — the handle for the background flush task
- Level filtering uses `tracing::Level` comparison against the configured `log_level`

### Dependencies
- SMET-T-0272 (API endpoint to POST to)
- `tracing-subscriber` already in dependencies

## Status Updates

*To be added during implementation*