---
id: refactor-machine-runner-crate-to
level: task
title: "Refactor Machine Runner Crate to Library API"
short_code: "SMET-T-0264"
created_at: 2026-03-28T16:52:35.868848+00:00
updated_at: 2026-03-28T17:30:22.542677+00:00
parent: SMET-I-0098
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0098
---

# Refactor Machine Runner Crate to Library API

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

Refactor the `cadre-machine-runner` crate from a standalone binary into a library with a public API that can be embedded by both the existing CLI binary and the new Tauri desktop app. The runner core must be controllable programmatically: start, stop, query status, update settings at runtime.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `apps/machine-runner/src/lib.rs` exports a `RunnerHandle` struct with methods: `start()`, `stop()`, `is_running() -> bool`, `status() -> RunnerStatus`, `update_settings(settings)`, `get_settings() -> Settings`
- [ ] `RunnerStatus` enum: `Registering`, `PendingApproval`, `Active { machine_id, connected, active_sessions }`, `Stopped`, `Error(String)`
- [ ] Runner lifecycle managed via `RunnerHandle` — calling `start()` spawns the registration + heartbeat loop as a background tokio task, `stop()` gracefully shuts it down
- [ ] `Settings` struct covers all ADR SMET-A-0003 settings: connection, behavior, repos, security, updates, logging
- [ ] `update_settings()` applies changes at runtime (e.g., changing `enabled` stops/starts the heartbeat, changing `repo_directories` triggers re-discovery)
- [ ] Existing `main.rs` binary refactored to use `RunnerHandle` — behavior unchanged
- [ ] All existing tests pass (`cargo test -p cadre-machine-runner`)
- [ ] Crate builds as both `lib` and `bin` targets

## Implementation Notes

### Technical Approach
- Split `Runner` into an inner core (the state machine) and `RunnerHandle` (the public API wrapper)
- `RunnerHandle` owns a `tokio::task::JoinHandle` for the background runner task and communicates via `mpsc` channels
- Settings struct is `Arc<RwLock<Settings>>` so the handle and inner runner can share it
- Status exposed via `tokio::sync::watch` channel — handle subscribes, runner publishes state changes
- Existing `main.rs` becomes a thin wrapper: load config → create handle → start → await shutdown signal

### Dependencies
- None — this is the foundation task

## Status Updates

*To be added during implementation*