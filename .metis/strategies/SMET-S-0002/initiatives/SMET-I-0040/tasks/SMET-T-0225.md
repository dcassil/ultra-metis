---
id: control-service-session-crud-api
level: task
title: "Control Service Session CRUD API"
short_code: "SMET-T-0225"
created_at: 2026-03-27T21:00:35.655039+00:00
updated_at: 2026-03-27T21:00:35.655039+00:00
parent: SMET-I-0040
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0040
---

# Control Service Session CRUD API

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Implement the core session CRUD endpoints in the Control Service: create a session, get session detail, and list sessions with filtering. These endpoints are the primary interface for the dashboard and runner to manage sessions.

## Acceptance Criteria

- [ ] `POST /api/sessions` — Creates a new session. Request body: `machine_id`, `repo_path`, `title`, `instructions`, `autonomy_level`, `work_item_id` (optional), `context` (optional). Validates machine exists, is trusted, and belongs to current user. Returns 201 with session ID and initial state `starting`. Stamps `user_id` from auth context.
- [ ] `GET /api/sessions/{id}` — Returns session detail: all fields plus computed `elapsed_time`, `last_activity` timestamp. Returns 404 if not found or not owned by current user.
- [ ] `GET /api/sessions` — List sessions with optional query params: `machine_id`, `repo_path`, `state`, `limit`, `offset`. Always scoped to `current_user`. Returns array with total count for pagination.
- [ ] Session creation validates machine is `trusted` (not `pending` or `revoked`) — returns 400 if machine not eligible
- [ ] Session creation inserts initial `session_event` record (null → starting)
- [ ] All endpoints require `DashboardAuth` (same pattern as machine endpoints)
- [ ] Unit tests for each endpoint covering success and error cases
- [ ] Existing machine tests continue to pass

## Implementation Notes

### Technical Approach
- Add routes in `routes.rs` following the existing pattern for machine endpoints
- `POST /api/sessions` handler: extract `DashboardAuth`, validate body, check machine ownership and trust status, insert session row, insert initial event, return created session
- `GET /api/sessions` handler: query with WHERE clauses for user_id and optional filters
- Use the same `ApiError` pattern for error responses
- Register new routes in `main.rs` router

### Dependencies
- SMET-T-0224 (Session Data Model) must be complete

## Status Updates

*To be added during implementation*