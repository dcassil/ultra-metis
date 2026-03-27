---
id: machine-runner-registration-and
level: task
title: "Machine Runner Registration and Heartbeat Client"
short_code: "SMET-T-0199"
created_at: 2026-03-27T16:18:41.939939+00:00
updated_at: 2026-03-27T16:18:41.939939+00:00
parent: SMET-I-0039
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0039
---

# Machine Runner Registration and Heartbeat Client

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Implement the client-side registration and heartbeat logic in the Machine Runner daemon (`apps/machine-runner/`). On startup, the runner registers itself with the Control Service, then enters a periodic heartbeat loop to maintain its online status. If the machine is in `pending` state, the runner enters a waiting mode and does not process commands until approved.

## Acceptance Criteria

- [ ] Machine Runner reads local configuration for: Control Service URL, machine name, machine API token (hardcoded static token for MVP), and heartbeat interval (default 20 seconds)
- [ ] On startup, runner calls `POST /machines/register` with machine name, platform info (OS, arch), capabilities JSON, and discovered repo list. Stores the returned machine ID locally for subsequent requests.
- [ ] If registration returns a `pending` status, runner enters a polling wait loop — retries heartbeat every interval and checks if status has changed to `trusted`
- [ ] Once trusted, runner transitions to active mode (ready to receive session commands in future initiatives)
- [ ] Heartbeat loop runs on a configurable interval (default 20s). Calls `POST /machines/{id}/heartbeat` with the current repo list.
- [ ] If heartbeat returns 401 (machine revoked), runner logs a warning and enters a stopped state — no further heartbeats or command processing
- [ ] If heartbeat fails due to network error, runner retries with exponential backoff (1s, 2s, 4s, 8s, max 60s) and logs each retry attempt
- [ ] Runner logs registration success/failure, heartbeat status changes, and state transitions at INFO level
- [ ] Unit tests for the registration client, heartbeat loop logic, and backoff behavior (using mock HTTP responses)

## Implementation Notes

### Technical Approach
- Use an async HTTP client (reqwest) for API calls to the Control Service
- The heartbeat loop runs as a tokio task spawned at startup after successful registration
- Machine configuration is loaded from a TOML or YAML config file at a well-known path (e.g., `~/.config/cadre/machine-runner.toml`) with fields: `control_service_url`, `machine_name`, `api_token`, `heartbeat_interval_secs`, `repo_directories`
- Platform info is auto-detected using `std::env::consts::OS` and `std::env::consts::ARCH`
- The runner maintains an internal state machine: `Registering -> Pending -> Active -> Stopped` (Stopped is entered on revocation or fatal error)
- The repo list sent during registration and heartbeat comes from the repo discovery module (SMET-T-0200)

### Runner State Machine

```
Registering --[success, pending]--> Pending --[approved]--> Active
Registering --[success, trusted]--> Active
Active --[401 on heartbeat]--> Stopped
Active --[network failure]--> Active (retry with backoff)
Pending --[401]--> Stopped
```

### Dependencies
- SMET-T-0198 (Control Service Machine Registry API) must be complete or testable — the runner is the client for those endpoints
- SMET-T-0200 (Machine Runner Repo Discovery) should be complete — registration and heartbeat include the repo list

### Risk Considerations
- Network resilience is critical — the runner must gracefully handle Control Service downtime without crashing
- The exponential backoff must have a reasonable cap (60s) to avoid long periods of silence
- Ensure the runner does not leak tokio tasks if it enters the Stopped state — cancel the heartbeat loop cleanly

## Status Updates

*To be added during implementation*