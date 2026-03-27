---
id: auth-middleware-user-context
level: task
title: "Auth Middleware & User Context Provider"
short_code: "SMET-T-0207"
created_at: 2026-03-27T16:28:40.210501+00:00
updated_at: 2026-03-27T19:09:24.609262+00:00
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

# Auth Middleware & User Context Provider

## Parent Initiative

[[SMET-I-0095]] — Control Dashboard Foundation

## Objective

Implement the MVP authentication middleware for the dashboard: a request interceptor that attaches a static bearer token to all outbound API requests, and a React context provider that exposes the current user identity throughout the component tree. This matches the Control Service's MVP auth model (static token maps to user_id=1). The architecture is designed so that when real authentication (JWT/OAuth) lands later, only the interceptor and context provider need to change — no downstream components are affected.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] An HTTP request interceptor (Axios or fetch wrapper) automatically adds `Authorization: Bearer static-token` to every outbound request to the Control Service
- [ ] The token value is configurable via environment variable (`VITE_AUTH_TOKEN`) with a sensible default of `static-token`
- [ ] A `UserContext` React context exists with a `UserProvider` component that wraps the app
- [ ] `useCurrentUser()` hook returns the current user object: `{ id: 1, name: "Default User" }`
- [ ] The `useCurrentUser()` hook has a TypeScript interface (`User`) that feature initiatives can extend
- [ ] The `UserProvider` is integrated into the app's component tree (wrapping the router/layout)
- [ ] Header component displays the current user's name (from context, not hardcoded in the header)
- [ ] If the interceptor receives a 401 response, it logs a warning to the console (placeholder for future redirect-to-login behavior)
- [ ] Unit tests verify the interceptor adds the auth header and the context provides user data

## Implementation Notes

### Technical Approach

1. Create `src/auth/types.ts`:
   - Define `User` interface: `{ id: number; name: string }`
   - Export for use across the app
2. Create `src/auth/UserContext.tsx`:
   - React context with `User | null` state
   - `UserProvider` component that provides the hardcoded MVP user
   - `useCurrentUser()` hook that consumes the context and throws if used outside provider
3. Create `src/auth/authInterceptor.ts`:
   - If using Axios: configure a request interceptor on the shared Axios instance that adds the `Authorization` header
   - If using fetch wrapper: create a `authFetch` wrapper that injects the header before delegating to `fetch`
   - Read token from `import.meta.env.VITE_AUTH_TOKEN` with fallback to `"static-token"`
   - Add a response interceptor that checks for 401 status and logs a warning
4. Wire `UserProvider` into `src/App.tsx`, wrapping the `RouterProvider` or `BrowserRouter`
5. Update `Header.tsx` (from SMET-T-0206) to display `user.name` from `useCurrentUser()`
6. Add to `.env.example`: `VITE_AUTH_TOKEN=static-token`

### Dependencies

- SMET-T-0205 (Project Scaffold) — needs the React/TypeScript foundation
- SMET-T-0206 (Application Shell) — the Header component where user name is displayed

### Design Decision: Swap Point

The auth middleware is intentionally the single swap point for authentication. When real auth arrives:
- Replace `authInterceptor.ts` to attach a JWT instead of static token
- Replace `UserProvider` to fetch user from an auth service/token claims
- All downstream components continue using `useCurrentUser()` unchanged

### File Structure Created

```
src/
  auth/
    types.ts
    UserContext.tsx
    authInterceptor.ts
    index.ts (barrel export)
```

## Status Updates

*To be added during implementation*