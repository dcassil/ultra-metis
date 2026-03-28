---
id: integration-tests-for-log-pipeline
level: task
title: "Integration Tests for Log Pipeline"
short_code: "SMET-T-0276"
created_at: 2026-03-28T17:50:00.377309+00:00
updated_at: 2026-03-28T17:50:00.377309+00:00
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

# Integration Tests for Log Pipeline

## Parent Initiative

[[SMET-I-0099]] — Machine-Level Debug Log Pipeline

## Objective

Write integration tests validating the full log pipeline: ingestion, query, SSE streaming, and level filtering.

## Acceptance Criteria

- [ ] Test: Post log batch, query back, verify all entries persisted with correct fields
- [ ] Test: Level filtering — post debug+info+warn+error logs, query with level=warn → only warn+error returned
- [ ] Test: Target filtering — query with target prefix → only matching entries
- [ ] Test: Pagination — post 10 logs, query with limit=3 → 3 returned, offset=3 → next 3
- [ ] Test: SSE stream delivers logs in real-time (post log, verify SSE client receives it)
- [ ] Test: Machine ownership validation — can't query another user's machine logs
- [ ] All tests pass alongside existing tests

## Implementation Notes

### Technical Approach
- Follow existing integration test patterns in `tests/integration.rs`
- Add helpers: `post_machine_logs`, `get_machine_logs`
- Reuse `register_and_approve_machine` helper
- SSE test optional (same caveat as session SSE — hard with reqwest)

### Dependencies
- SMET-T-0272 (API endpoints)

## Status Updates

*To be added during implementation*