---
id: machine-runner-output-capture-and
level: task
title: "Machine Runner Output Capture and Event Emission"
short_code: "SMET-T-0247"
created_at: 2026-03-28T00:36:55.970659+00:00
updated_at: 2026-03-28T00:53:54.688904+00:00
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

# Machine Runner Output Capture and Event Emission

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Implement the Machine Runner's output capture pipeline: read stdout/stderr from Claude Code processes line by line, classify output, detect approval requests, and emit typed events to the Control Service.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `output_capture.rs` module in machine-runner with `OutputCapture` struct
- [ ] Reads stdout/stderr from the spawned process via piped handles (update supervisor to provide pipe access)
- [ ] Classifies each line: `info` (default), `warning` (lines containing "warning"/"WARN"), `error` (stderr or lines containing "error"/"ERROR"), `summary` (final output blocks)
- [ ] Approval request detection: recognizes Claude Code structured output patterns for tool approval prompts (JSON with `type: "tool_use"` or similar markers)
- [ ] Each classified line emitted as a typed event via `POST /api/sessions/{id}/events` to the control service
- [ ] Approval requests emitted as `ApprovalRequest` events with question text and options extracted from structured output
- [ ] Events include monotonically increasing sequence numbers for ordering
- [ ] Runner `client.rs` extended with `post_session_event(session_id, event)` method
- [ ] Batching: events buffered and sent in batches (up to 10 or every 500ms, whichever comes first) to reduce HTTP overhead
- [ ] Unit tests for output classification and approval detection

## Implementation Notes

### Technical Approach
- Supervisor's `start_session` already captures stdout/stderr via `Stdio::piped()` — provide access to the pipes via the process handle
- OutputCapture spawns a tokio task that reads `BufReader::lines()` from stdout and stderr
- Each line is classified and batched into events
- Approval detection: look for JSON output from Claude Code containing tool approval prompts
- Batch sender: `tokio::time::interval(500ms)` flushes the batch, or immediate flush when batch hits 10 items

### Dependencies
- SMET-T-0246 (Event Model), existing supervisor.rs

## Status Updates

*To be added during implementation*