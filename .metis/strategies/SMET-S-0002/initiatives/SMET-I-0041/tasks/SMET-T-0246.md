---
id: session-event-model-and
level: task
title: "Session Event Model and Persistence Schema"
short_code: "SMET-T-0246"
created_at: 2026-03-28T00:36:55.112850+00:00
updated_at: 2026-03-28T00:42:04.224337+00:00
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

# Session Event Model and Persistence Schema

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Define the typed event model for session activity streams and extend the database schema to persist events for live monitoring and future history/replay.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SessionEventType` enum: `OutputLine`, `ApprovalRequest`, `ApprovalResponse`, `GuidanceInjected`, `StateChanged`, `PolicyViolation`
- [ ] `OutputCategory` enum: `Info`, `Warning`, `Error`, `Summary`
- [ ] `InjectionType` enum: `Normal`, `SideNote`, `Interrupt`
- [ ] `session_output_events` table: `id`, `session_id`, `event_type`, `category` (for output lines), `content` (text), `metadata` (JSON), `sequence_num` (auto-increment for ordering), `timestamp`
- [ ] `pending_approvals` table: `id`, `session_id`, `question`, `options` (JSON array), `context`, `status` (pending/responded/expired), `response_choice`, `response_note`, `created_at`, `responded_at`
- [ ] Event types implement `Serialize`/`Deserialize` with roundtrip tests
- [ ] Request/response types for event ingestion API (`IngestEventRequest`)
- [ ] Existing tests pass

## Implementation Notes

### Technical Approach
- `session_output_events` is separate from `session_events` (state transitions) — this table is high-volume output lines
- `sequence_num` uses INTEGER PRIMARY KEY AUTOINCREMENT pattern for guaranteed ordering
- `pending_approvals` tracks approval requests awaiting user response — when responded, status changes and response fields populated
- Metadata JSON stores event-type-specific data (e.g., exit code for state changes, subagent ID for SDD events)

### Dependencies
- Existing session model from SMET-I-0040

## Status Updates

*To be added during implementation*