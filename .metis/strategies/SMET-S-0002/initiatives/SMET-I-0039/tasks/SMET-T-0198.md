---
id: control-service-machine-registry
level: task
title: "Control Service Machine Registry API"
short_code: "SMET-T-0198"
created_at: 2026-03-27T16:18:40.628462+00:00
updated_at: 2026-03-27T16:18:40.628462+00:00
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

# Control Service Machine Registry API

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Implement the six REST API endpoints in the Control Service (`apps/control-api/`) that form the machine registry: register, heartbeat, list, detail, approve, and revoke. These endpoints are the server-side counterpart to the Machine Runner client and the data source for the Control Dashboard.

## Acceptance Criteria

- [ ] `POST /machines/register` — accepts `name`, `platform`, `capabilities` (JSON), and a list of `repos` (name+path pairs). Creates a new machine in `pending` status. Returns the machine ID. Requires a valid machine API token in the `Authorization` header. Scopes the machine to the token's `user_id`.
- [ ] `POST /machines/{id}/heartbeat` — updates `last_heartbeat` to current timestamp. Accepts an optional updated `repos` list (upserts repo records). Returns 401 if machine is revoked. Returns 403 if machine is still `pending`.
- [ ] `GET /machines` — returns all machines for the current user (`WHERE user_id = :current_user`). Each machine includes: id, name, platform, status, trust_tier, computed connectivity status (online/stale/offline based on last_heartbeat), active_session_count (hardcoded 0 for now), last_heartbeat timestamp.
- [ ] `GET /machines/{id}` — returns full machine detail including metadata, trust_tier, list of repos, computed connectivity status, and active sessions (empty array for now). Returns 404 if machine does not belong to current user.
- [ ] `POST /machines/{id}/approve` — transitions machine from `pending` to `trusted`. Sets trust_tier to `trusted` by default. Returns 409 if machine is not in `pending` status. Validates machine belongs to current user.
- [ ] `POST /machines/{id}/revoke` — transitions machine to `revoked` status regardless of current status. Returns 404 if machine not found or not owned by current user.
- [ ] All endpoints return appropriate HTTP status codes (201 for register, 200 for others, 401/403/404/409 for errors) with JSON error bodies
- [ ] Unit tests for each endpoint handler covering happy path and error cases

## Implementation Notes

### Technical Approach
- Implement as route handlers in the Control Service web framework (Axum or Actix-web, depending on what the monorepo restructure establishes)
- Machine status computation (online/stale/offline) is a shared utility from the domain types (SMET-T-0197) — call it when serializing machine responses
- The `register` endpoint inserts into both `machines` and `machine_repos` tables in a transaction
- The `heartbeat` endpoint does an upsert on `machine_repos` (delete removed repos, insert new ones, update existing) plus updates `last_heartbeat`
- Active session count and session list are placeholder fields (return 0 and [] respectively) — they will be populated when SMET-I-0040 (Session Management) is implemented
- Request validation: reject missing/malformed fields with 400 and descriptive error messages

### API Endpoints Detail

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/machines/register` | Machine token | Register new machine |
| POST | `/machines/{id}/heartbeat` | Machine token | Update liveness |
| GET | `/machines` | User auth | List user's machines |
| GET | `/machines/{id}` | User auth | Machine detail |
| POST | `/machines/{id}/approve` | User auth | Approve pending machine |
| POST | `/machines/{id}/revoke` | User auth | Revoke a machine |

### Dependencies
- SMET-T-0197 (Machine Data Model and Database Schema) must be complete — this task builds on those tables and Rust types
- SMET-T-0203 (Multi-Tenancy Scaffolding and Auth Middleware) should be complete or in progress — the endpoints depend on `request.user_id` being set by middleware

### Risk Considerations
- The heartbeat repo-upsert logic is the most complex part — needs careful transaction handling to avoid partial updates
- Ensure all list/detail queries are scoped by `user_id` to prevent cross-tenant data leaks even in MVP mode

## Status Updates

*To be added during implementation*