---
id: control-api-planning-data-endpoints
level: task
title: "Control API Planning Data Endpoints"
short_code: "SMET-T-0240"
created_at: 2026-03-28T00:33:37.708305+00:00
updated_at: 2026-03-28T00:54:05.319355+00:00
parent: SMET-I-0071
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0071
---

# Control API Planning Data Endpoints

## Parent Initiative

[[SMET-I-0071]] Planning Data Views in Control Dashboard

## Objective

Add REST endpoints to the control-api (Rust/Axum) that expose cadre planning data to the dashboard. The control-api currently only serves machine, session, and policy data. This task adds a `/api/planning/*` route group that reads from the cadre-store (file-based .metis directory) and returns document, hierarchy, rule, and quality data as JSON.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] cadre-core and cadre-store added as dependencies to control-api Cargo.toml
- [ ] Planning module (`planning.rs`) created with Store initialization from configurable .metis path
- [ ] `GET /api/planning/documents` — list documents with optional type/phase query filters
- [ ] `GET /api/planning/documents/:short_code` — read single document with full content and metadata
- [ ] `GET /api/planning/documents/search?q=` — full-text search across documents
- [ ] `GET /api/planning/hierarchy` — returns the full document tree (parent-child relationships)
- [ ] `GET /api/planning/rules` — list active rules with optional scope filter
- [ ] `GET /api/planning/quality/:short_code` — quality records for a document
- [ ] All endpoints return proper JSON responses with error handling
- [ ] Integration test covering each endpoint with a temporary .metis project
- [ ] `cargo build` and `cargo test` pass for control-api

## Implementation Notes

### Technical Approach
- Add `cadre-core` and `cadre-store` workspace dependencies to `apps/control-api/Cargo.toml`
- Create `src/planning.rs` module with route handlers
- Initialize a `cadre_store::Store` from a configurable project path (env var `CADRE_PROJECT_PATH` or default)
- Store the `Store` instance in `AppState` (wrapped in Arc)
- Add `/api/planning` route group in `lib.rs` `build_app()`
- Response types: create planning-specific response structs in `models.rs` or a new `planning_models.rs`
- Hierarchy endpoint: load all documents, build tree from parent references

### Dependencies
- cadre-core and cadre-store crates (already in workspace)
- No other tasks block this

### Files to Modify
- `apps/control-api/Cargo.toml` — add deps
- `apps/control-api/src/lib.rs` — add planning routes, update AppState
- `apps/control-api/src/planning.rs` — new file, route handlers
- `apps/control-api/src/models.rs` or new `planning_models.rs` — response types
- `apps/control-api/tests/` — integration tests

## Status Updates

### 2026-03-28: Implementation Complete
- Added `cadre-core` and `cadre-store` dependencies to control-api Cargo.toml
- Created `PlanningState` struct in lib.rs with optional `Arc<DocumentStore>`
- Created `planning.rs` module with 6 route handlers:
  - `GET /api/planning/documents` — list with type/phase/parent filters
  - `GET /api/planning/documents/search` — full-text search with type filter and limit
  - `GET /api/planning/documents/:short_code` — detail with children list and raw content
  - `GET /api/planning/hierarchy` — recursive tree built from parent references
  - `GET /api/planning/rules` — list rules_config documents with scope filter
  - `GET /api/planning/quality/:short_code` — quality records for a document
- Added `build_app_with_planning()` and `init_planning_state()` for test/production flexibility
- Planning routes mounted at `/api/planning` as a nested router with separate state
- Created 10 integration tests covering all endpoints (list, filter, detail, not-found, search, hierarchy, rules, quality)
- All 39 tests pass (29 existing + 10 new), zero warnings, no regressions