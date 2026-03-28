---
id: machine-runner-guidance-injection
level: task
title: "Machine Runner Guidance Injection and Approval Forwarding"
short_code: "SMET-T-0250"
created_at: 2026-03-28T00:36:59.208237+00:00
updated_at: 2026-03-28T01:02:09.704455+00:00
parent: SMET-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0041
---

# Machine Runner Guidance Injection and Approval Forwarding

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Implement the Machine Runner's handling of guidance injection and approval forwarding commands received from the Control Service.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Runner `process_command()` handles `respond` command: extracts approval_id and choice, writes response to Claude Code process stdin (or via hook mechanism)
- [ ] Runner `process_command()` handles `inject` command: extracts message and injection_type, writes to process stdin
- [ ] Normal injection: message written to stdin as-is
- [ ] Side-note injection: message prefixed with context marker (e.g., `[Note from user]: ...`)
- [ ] Interrupt injection: sends SIGUSR1 or writes a special marker that Claude Code recognizes as urgent
- [ ] After forwarding injection/response, runner emits confirmation event back to control service
- [ ] If process stdin is closed (process exited), runner reports error to control service
- [ ] Unit tests for command parsing and injection formatting

## Implementation Notes

### Technical Approach
- Supervisor needs to provide write access to the child process stdin — update `ProcessHandle` to store `ChildStdin`
- For MVP, all injection goes through stdin. More sophisticated hook-based injection is post-MVP.
- Approval response forwarding: write the choice to stdin so Claude Code proceeds
- Injection events emitted via `post_session_event` to control service for audit trail

### Dependencies
- SMET-T-0249 (Intervention API — queues the commands), SMET-T-0247 (Output Capture — shares pipe infrastructure)

## Status Updates

*To be added during implementation*