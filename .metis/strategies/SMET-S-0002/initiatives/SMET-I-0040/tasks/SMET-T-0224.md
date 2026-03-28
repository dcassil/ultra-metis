---
id: session-data-model-and-database
level: task
title: "Session Data Model and Database Schema"
short_code: "SMET-T-0224"
created_at: 2026-03-27T21:00:34.871701+00:00
updated_at: 2026-03-27T23:50:01.372716+00:00
parent: SMET-I-0040
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0040
---

# Session Data Model and Database Schema

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Define the session data model and extend the SQLite schema in `apps/control-api/` to support sessions. This is the foundation all other session tasks build on — the state enum, database tables, and Rust model types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SessionState` enum defined in `models.rs`: `Starting`, `Running`, `WaitingForInput`, `Paused`, `Completed`, `Failed`, `Stopped`
- [ ] `SessionRow` struct in `models.rs` with fields: `id`, `user_id`, `machine_id`, `repo_path`, `title`, `instructions`, `autonomy_level`, `work_item_id` (optional), `context` (optional JSON), `state`, `created_at`, `updated_at`, `started_at`, `completed_at`
- [ ] `AutonomyLevel` enum: `Normal`, `Stricter`, `Autonomous`
- [ ] `sessions` table added to `db.rs` schema init with all columns, foreign keys to `machines` and `users`
- [ ] `session_events` table for state transition history: `id`, `session_id`, `from_state`, `to_state`, `timestamp`, `metadata` (JSON)
- [ ] `session_commands` table for queued commands from control service to runner: `id`, `session_id`, `command_type`, `payload` (JSON), `status` (pending/delivered/executed), `created_at`, `delivered_at`
- [ ] All new types implement `Serialize`/`Deserialize` and have `#[cfg(test)]` unit tests
- [ ] Existing tests continue to pass (`cargo test -p control-api`)

## Implementation Notes

### Technical Approach
- Follow the existing pattern in `models.rs` where `MachineRow`, `MachineStatus`, `TrustTier` are defined
- `SessionState` should implement `Display`, `FromStr`, and serialize as lowercase strings for SQLite storage
- `session_commands` table is the mechanism for control service → runner communication
- `autonomy_level` stored as TEXT in SQLite, same pattern as `status` and `trust_tier` on machines
- `work_item_id` is a free-form string (Cadre short code like `SMET-T-0123`) — no foreign key
- `context` field is JSON text for optional preloaded notes, constraints, architecture guidance

### Dependencies
- Existing `db.rs` schema and `models.rs` types in `apps/control-api/`

## Status Updates

*To be added during implementation*