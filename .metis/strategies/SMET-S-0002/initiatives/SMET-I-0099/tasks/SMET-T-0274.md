---
id: tracing-integration-for-tauri-and
level: task
title: "Tracing Integration for Tauri and Headless Modes"
short_code: "SMET-T-0274"
created_at: 2026-03-28T17:49:58.562908+00:00
updated_at: 2026-03-28T17:49:58.562908+00:00
parent: SMET-I-0099
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0099
---

# Tracing Integration for Tauri and Headless Modes

## Parent Initiative

[[SMET-I-0099]] — Machine-Level Debug Log Pipeline

## Objective

Wire the custom tracing layer into both the Tauri desktop app and the headless CLI binary, with appropriate output destinations for each mode.

## Acceptance Criteria

- [ ] **Headless CLI** (`main.rs`): tracing subscriber composed of: `EnvFilter::from_default_env()` + `fmt::Layer` (stderr) + `LogForwardingLayer` (API). RUST_LOG env var controls stderr output. Settings `log_level` controls API forwarding.
- [ ] **Tauri desktop** (`runner-desktop/src-tauri/main.rs`): tracing subscriber composed of: `LogForwardingLayer` (API) + file appender layer (writes to `~/.config/cadre/runner.log`). No stderr since there's no terminal.
- [ ] Local log file rotated: max 10MB, keep 3 old files (use `tracing-appender` crate)
- [ ] `RunnerHandle::start()` initializes the log forwarding layer when the runner starts (needs machine_id from registration)
- [ ] Log forwarding only active when runner is registered and has a machine_id — before registration, logs go to local only
- [ ] Changing `log_level` in settings takes effect on the forwarding layer (dynamic filter)
- [ ] All existing runner tests pass

## Implementation Notes

### Technical Approach
- Use `tracing_subscriber::Registry` with `.with()` to compose multiple layers
- `tracing-appender` for file rotation in Tauri mode
- `EnvFilter` for RUST_LOG support in headless mode
- The forwarding layer starts in "disabled" state and activates once `machine_id` is known (after registration)
- Dynamic level filtering via `tracing_subscriber::reload` or by checking the settings in the layer's `enabled()` method

### Dependencies
- SMET-T-0273 (LogForwardingLayer)
- `tracing-appender` crate for file logging

## Status Updates

*To be added during implementation*