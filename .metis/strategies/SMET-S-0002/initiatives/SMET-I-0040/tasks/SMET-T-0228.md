---
id: machine-runner-process-supervisor
level: task
title: "Machine Runner Process Supervisor"
short_code: "SMET-T-0228"
created_at: 2026-03-27T21:00:38.334647+00:00
updated_at: 2026-03-28T00:07:17.504966+00:00
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

# Machine Runner Process Supervisor

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Implement the process supervisor in the Machine Runner that spawns, monitors, and controls Claude Code AI processes for sessions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `supervisor.rs` module in `apps/machine-runner/src/` with `ProcessSupervisor` struct
- [ ] `start_session()` — Spawns Claude Code process in configured repo directory with instructions. Captures stdout/stderr. Returns session handle.
- [ ] `stop_session(session_id)` — SIGTERM, wait 10s, then SIGKILL if still running
- [ ] `force_stop_session(session_id)` — SIGKILL immediately
- [ ] `pause_session(session_id)` — SIGSTOP to suspend process
- [ ] `resume_session(session_id)` — SIGCONT to resume suspended process
- [ ] Process exit monitoring: detects clean/crash exit, captures exit code
- [ ] Active session tracking: `HashMap<SessionId, ProcessHandle>` with PID, start time, state
- [ ] Graceful shutdown: on runner SIGTERM, stops all active sessions before exiting
- [ ] Autonomy level maps to CLI flags (`--allowedTools` for stricter, `--dangerously-skip-permissions` for autonomous)
- [ ] Working directory set to session's `repo_path`
- [ ] Unit tests for supervisor state tracking

## Implementation Notes

### Technical Approach
- `tokio::process::Command` for async process spawning
- Supervisor as tokio task, receiving commands via `mpsc` channel
- `child.wait()` for process monitoring
- Claude Code invocation: `claude --print --output-format json -p "<instructions>"`
- MVP: one session per machine at a time
- `nix::sys::signal` crate for SIGSTOP/SIGCONT

### Dependencies
- SMET-T-0227 (Command Routing), `nix` crate

## Status Updates

*To be added during implementation*