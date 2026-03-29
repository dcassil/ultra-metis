---
id: fix-invalid-injection-event-type
level: task
title: "Fix Invalid injection Event Type in Machine Runner"
short_code: "SMET-T-0286"
created_at: 2026-03-29T01:29:13.035288+00:00
updated_at: 2026-03-29T01:31:50.832492+00:00
parent: SMET-I-0102
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0102
---

# Fix Invalid injection Event Type in Machine Runner

## Parent Initiative

[[SMET-I-0102]]

## Objective

Fix the machine runner's injection confirmation event which uses `event_type: "injection"` — a value not in the `SessionOutputEventType` enum. This causes the API to reject the entire event batch with a deserialization error.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Runner injection confirmation uses a valid `SessionOutputEventType` value
- [ ] Existing `output_capture` tests pass
- [ ] `cargo test` passes for machine-runner crate
- [ ] `make build` succeeds

## Implementation Notes

### Technical Approach

In `apps/machine-runner/src/runner.rs` line 874, change:
```rust
event_type: "injection".to_string(),
```
to:
```rust
event_type: "output_line".to_string(),
```

The category is already `"info"` and the content describes the injection, so using `output_line` is appropriate. The dashboard already receives a separate `guidance_injected` event from the API's `inject_guidance` endpoint.

### Key Files
- `apps/machine-runner/src/runner.rs:874`

## Status Updates

*To be added during implementation*