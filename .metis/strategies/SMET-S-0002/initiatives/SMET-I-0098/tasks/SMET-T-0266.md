---
id: settings-persistence-and-os
level: task
title: "Settings Persistence and OS Keychain Integration"
short_code: "SMET-T-0266"
created_at: 2026-03-28T16:52:37.583232+00:00
updated_at: 2026-03-28T17:38:17.514525+00:00
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

# Settings Persistence and OS Keychain Integration

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

Implement the settings persistence layer with the full ADR SMET-A-0003 settings model, and integrate OS keychain for secure API token storage. Settings must be readable/writable from both the Rust backend and the React frontend via Tauri IPC.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Settings` struct in runner crate covers all fields: connection (url, name), behavior (auto_start, start_minimized, enabled, heartbeat_interval, max_concurrent_sessions), repos (directories, allowed, blocked, restrict_to_repos), security (local_approval_required, allowed_autonomy_levels, block_autonomous_mode, session_timeout_minutes, action categories), updates (auto_update, update_channel), logging (log_level)
- [ ] Settings persisted to `~/.config/cadre/settings.json` (JSON, not TOML — easier for programmatic read/write)
- [ ] Migration: if old `machine-runner.toml` exists, auto-migrate to new settings.json on first launch
- [ ] API token stored in OS keychain via `keyring` crate (macOS Keychain, Windows Credential Manager, Linux libsecret)
- [ ] Fallback: if keychain unavailable (headless/CI), store token in settings file with a warning logged
- [ ] Tauri IPC commands: `get_settings`, `save_settings`, `get_token`, `set_token`, `delete_token`
- [ ] Settings changes trigger `RunnerHandle::update_settings()` so runner applies them immediately
- [ ] Unit tests for settings serialization/deserialization roundtrip
- [ ] Unit tests for TOML → JSON migration

## Implementation Notes

### Technical Approach
- `keyring` crate (v3) with service name `cadre-machine-runner`, username = machine_name
- Settings JSON schema versioned: `{"version": 1, "connection": {...}, "behavior": {...}, ...}` for future migration
- Tauri commands wrap the settings + keyring operations, return Results so frontend can show errors
- Settings file created with sensible defaults on first run if missing

### Dependencies
- SMET-T-0264 (Runner Library API — `Settings` struct and `update_settings`)
- `keyring` crate v3

## Status Updates

*To be added during implementation*