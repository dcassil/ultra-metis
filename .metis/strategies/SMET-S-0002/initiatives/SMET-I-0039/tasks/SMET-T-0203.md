---
id: multi-tenancy-scaffolding-and-auth
level: task
title: "Multi-Tenancy Scaffolding and Auth Middleware"
short_code: "SMET-T-0203"
created_at: 2026-03-27T16:18:45.779031+00:00
updated_at: 2026-03-27T16:18:45.779031+00:00
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

# Multi-Tenancy Scaffolding and Auth Middleware

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Implement the MVP authentication middleware and multi-tenancy data scoping for the Control Service. In MVP mode, all requests are attributed to a single default user (user_id=1). The middleware is designed as a swappable layer so that replacing it with JWT-based auth later requires no changes to endpoint handlers.

## Acceptance Criteria

- [ ] Auth middleware is registered in the Control Service request pipeline and runs before all route handlers
- [ ] MVP mode: middleware unconditionally sets `request.user_id = 1` for all dashboard/user-facing requests (no actual authentication check)
- [ ] Machine token auth: for machine-facing endpoints (`/machines/register`, `/machines/{id}/heartbeat`), middleware reads the `Authorization: Bearer <token>` header, looks up the token in a `machine_tokens` table, and resolves it to a `user_id`. In MVP, a single static token is seeded that maps to `user_id=1`.
- [ ] `machine_tokens` table created with columns: `id` (UUID PK), `token` (text, unique, indexed), `user_id` (FK to users), `created_at`. Seed migration inserts one default token.
- [ ] The resolved `user_id` is available to all route handlers via a request extension or extractor (e.g., Axum's `Extension<AuthContext>` or similar pattern)
- [ ] All machine queries include `WHERE user_id = :current_user` — never return machines belonging to other users
- [ ] If a machine token is invalid or missing on machine-facing endpoints, return 401 Unauthorized with a JSON error body
- [ ] The middleware design is documented with a clear comment/doc explaining where to swap in JWT verification for production auth
- [ ] Unit tests: verify middleware sets user_id correctly, verify invalid token returns 401, verify user scoping in queries

## Implementation Notes

### Technical Approach
- Create an `AuthContext` struct: `{ user_id: i64, org_id: i64, team_id: i64 }` — populated by middleware
- Two middleware paths:
  1. **Dashboard requests** (GET /machines, GET /machines/{id}, POST /machines/{id}/approve, POST /machines/{id}/revoke): MVP sets `AuthContext { user_id: 1, org_id: 1, team_id: 1 }` unconditionally. The swap point for JWT auth is clearly marked.
  2. **Machine requests** (POST /machines/register, POST /machines/{id}/heartbeat): reads `Authorization` header, queries `machine_tokens` table, resolves to `AuthContext`. Returns 401 if token not found.
- The `machine_tokens` seed: `INSERT INTO machine_tokens (id, token, user_id) VALUES (gen_uuid(), 'cadre-mvp-static-token', 1)` — the Machine Runner config uses this token
- All repository/query functions accept `user_id` as a parameter, ensuring scoping is enforced at the data layer, not just the middleware layer

### Auth Flow Diagram

```
Request -> Auth Middleware -> [Dashboard path: set user_id=1]
                           -> [Machine path: read Bearer token -> lookup -> set user_id]
         -> Route Handler (receives AuthContext)
         -> Database Query (WHERE user_id = :auth_context.user_id)
```

### Dependencies
- SMET-T-0197 (Machine Data Model and Database Schema) — the `machine_tokens` table and seed data build on that migration
- This task should be completed before or in parallel with SMET-T-0198 (Control Service Machine Registry API) since the API endpoints depend on `AuthContext`

### Risk Considerations
- The static MVP token must be clearly documented as development-only and should not be used in production
- Ensure the middleware cannot be bypassed — all routes must go through the auth layer
- The `AuthContext` pattern must be consistent so that swapping in JWT auth is truly a single-file change

## Status Updates

*To be added during implementation*