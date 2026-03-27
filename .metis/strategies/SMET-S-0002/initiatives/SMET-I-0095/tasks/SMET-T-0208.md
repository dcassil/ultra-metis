---
id: api-client-layer
level: task
title: "API Client Layer"
short_code: "SMET-T-0208"
created_at: 2026-03-27T16:28:41.042782+00:00
updated_at: 2026-03-27T19:13:44.463638+00:00
parent: SMET-I-0095
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0095
---

# API Client Layer

## Parent Initiative

[[SMET-I-0095]] — Control Dashboard Foundation

## Objective

Build the typed HTTP client layer that all feature initiatives use to communicate with the Control Service. This includes the base Axios/fetch instance configured with the correct base URL, the auth interceptor integration, centralized error handling, and stub API modules grouped by domain (machines, sessions, monitoring, history, policies). After this task, any feature initiative can import a typed API client and start making calls without worrying about auth, base URL, or error handling.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A shared HTTP client instance (Axios or fetch wrapper) is configured in `src/api/client.ts`
- [ ] Base URL reads from `import.meta.env.VITE_API_BASE_URL` with a default of `http://localhost:3000`
- [ ] The auth interceptor from SMET-T-0207 is wired into the shared client instance
- [ ] Centralized error handling intercepts all responses and categorizes errors:
  - Network errors (no response) — `NetworkError`
  - 401 Unauthorized — `AuthError` (logged, placeholder for redirect)
  - 4xx Client errors — `ClientError` with status and message
  - 5xx Server errors — `ServerError` with status and message
- [ ] Custom error types are defined in `src/api/errors.ts` with TypeScript discriminated unions
- [ ] Stub API modules exist for each domain: `src/api/machines.ts`, `src/api/sessions.ts`, `src/api/monitoring.ts`, `src/api/history.ts`, `src/api/policies.ts`
- [ ] Each stub module exports typed function signatures (with `// TODO: implement in SMET-I-00XX` comments) so feature initiatives know exactly what to fill in
- [ ] A `src/api/health.ts` module exports `checkHealth(): Promise<HealthStatus>` that calls `GET /health` on the Control Service
- [ ] `HealthStatus` type is defined: `{ status: "ok" | "degraded" | "error"; version?: string; uptime?: number }`
- [ ] Barrel export in `src/api/index.ts` re-exports all modules for clean imports
- [ ] Unit tests verify error categorization logic (mock different HTTP status codes)

## Implementation Notes

### Technical Approach

1. Install Axios: `npm install axios`
2. Create `src/api/client.ts`:
   - Create an Axios instance with `baseURL` from env var
   - Attach the auth request interceptor from `src/auth/authInterceptor.ts`
   - Attach a response error interceptor that wraps errors into typed custom errors
3. Create `src/api/errors.ts`:
   - Define error classes: `ApiError` (base), `NetworkError`, `AuthError`, `ClientError`, `ServerError`
   - Each includes relevant metadata (status code, response body, original error)
   - Use discriminated union pattern for exhaustive error handling in consumers
4. Create `src/api/health.ts`:
   - `checkHealth()` function that calls `GET /health`
   - Returns a typed `HealthStatus` object
   - Catches errors and returns `{ status: "error" }` on failure
5. Create stub API modules for each domain:
   - `src/api/machines.ts` — `listMachines()`, `getMachine(id)`, `updateMachine(id, data)` stubs
   - `src/api/sessions.ts` — `listSessions()`, `getSession(id)`, `createSession(data)`, `stopSession(id)` stubs
   - `src/api/monitoring.ts` — `getMetrics(machineId)`, `getAlerts()` stubs
   - `src/api/history.ts` — `listHistoryEntries(filters)`, `getHistoryEntry(id)` stubs
   - `src/api/policies.ts` — `listPolicies()`, `getPolicy(id)`, `createPolicy(data)`, `updatePolicy(id, data)` stubs
   - Each function throws `new Error("Not implemented — see SMET-I-00XX")` for now
6. Create `src/api/index.ts` barrel that re-exports everything

### Dependencies

- SMET-T-0205 (Project Scaffold) — needs the React/TypeScript foundation
- SMET-T-0207 (Auth Middleware) — the auth interceptor is wired into the client

### Risk Considerations

- The stub API types are best guesses based on the initiative descriptions. Feature initiatives may need to adjust these interfaces when they implement the real endpoints. That is expected — the stubs serve as documentation of the intended shape, not a rigid contract.

### File Structure Created

```
src/
  api/
    client.ts       (shared Axios instance)
    errors.ts       (custom error types)
    health.ts       (health check endpoint)
    machines.ts     (stub)
    sessions.ts     (stub)
    monitoring.ts   (stub)
    history.ts      (stub)
    policies.ts     (stub)
    index.ts        (barrel export)
```

## Status Updates

*To be added during implementation*