---
id: command-routing-via-heartbeat
level: task
title: "Command Routing via Heartbeat Polling"
short_code: "SMET-T-0227"
created_at: 2026-03-27T21:00:37.514186+00:00
updated_at: 2026-03-27T23:59:17.261755+00:00
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

# Command Routing via Heartbeat Polling

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Implement the command routing mechanism between the Control Service and Machine Runner. The runner polls for pending commands during its heartbeat cycle, and the control service delivers queued commands.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Heartbeat response extended to include `pending_commands` array: `command_id`, `command_type` (start_session, stop, force_stop, pause, resume), `payload` (JSON)
- [ ] `GET /api/machines/{id}/commands` — Returns pending commands for a machine. Requires `MachineTokenAuth`.
- [ ] `POST /api/machines/{id}/commands/{cmd_id}/ack` — Runner acknowledges receipt, marking command as `delivered`. Requires `MachineTokenAuth`.
- [ ] Session creation inserts a `start_session` command with payload containing `session_id`, `repo_path`, `title`, `instructions`, `autonomy_level`, `context`
- [ ] Machine Runner `client.rs` updated: after heartbeat, fetch pending commands, process each, acknowledge receipt
- [ ] Machine Runner `runner.rs` updated: command dispatch loop routing received commands to handlers
- [ ] Commands not acknowledged within 5 minutes are re-delivered on next poll
- [ ] Unit tests for command queue operations and runner command dispatch

## Implementation Notes

### Technical Approach
- Piggyback on existing heartbeat cycle: after heartbeat, runner calls `GET /commands` and processes pending commands
- Commands use simple SQLite queue: insert `pending`, runner fetches and acks to `delivered`, marks `executed` after state report
- Polling interval is heartbeat interval (15-30s) — acceptable for MVP

### Dependencies
- SMET-T-0224 (session_commands table), SMET-T-0226 (control actions insert commands)
- Existing heartbeat in runner `client.rs` and `runner.rs`

## Status Updates

*To be added during implementation*