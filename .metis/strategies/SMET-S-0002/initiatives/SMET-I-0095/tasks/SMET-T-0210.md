---
id: health-check-build-pipeline
level: task
title: "Health Check & Build Pipeline"
short_code: "SMET-T-0210"
created_at: 2026-03-27T16:28:42.539237+00:00
updated_at: 2026-03-27T19:13:44.939946+00:00
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

# Health Check & Build Pipeline

## Parent Initiative

[[SMET-I-0095]] — Control Dashboard Foundation

## Objective

Implement two final pieces of the dashboard foundation: (1) a health check system that polls the Control Service and displays connection status in the header, and (2) a production build pipeline with static hosting configuration. After this task, users see a live connection indicator in the dashboard header, and the application is ready for deployment as a static SPA.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A `useHealthCheck()` React hook polls `GET /health` on the Control Service at startup and every 30 seconds thereafter
- [ ] The hook returns a `ConnectionStatus` value: `"connected"` (200 OK), `"degraded"` (non-200 but reachable), or `"disconnected"` (network error/timeout)
- [ ] A `ConnectionIndicator` component renders in the header bar with color-coded status: green dot + "Connected", yellow dot + "Degraded", red dot + "Disconnected"
- [ ] When status is `"disconnected"`, a dismissible banner appears below the header explaining the issue: "Unable to reach the Control Service. Check that it is running at {baseUrl}."
- [ ] The banner prevents (disables) action buttons across the app when disconnected (via a `useIsOnline()` hook that reads from health check context)
- [ ] Health check uses the API client from SMET-T-0208 (`checkHealth()` from `src/api/health.ts`)
- [ ] `npm run build` produces an optimized production bundle in `dist/` with hashed asset filenames for cache busting
- [ ] A `_redirects` file (for Netlify) or equivalent SPA fallback configuration is included in `public/` to handle client-side routing (all paths serve `index.html`)
- [ ] A `Dockerfile` (optional, stretch) or `nginx.conf` snippet is provided for containerized static hosting with SPA fallback
- [ ] Production build has no console warnings or errors
- [ ] Build output size is documented (baseline for future monitoring)
- [ ] End-to-end smoke test: dev server starts, all routes navigate, health check indicator shows status, build completes cleanly

## Implementation Notes

### Technical Approach

1. Create `src/hooks/useHealthCheck.ts`:
   - Uses `useEffect` with `setInterval` to poll health endpoint every 30 seconds
   - Calls `checkHealth()` from `src/api/health.ts`
   - Returns `{ status: ConnectionStatus; lastChecked: Date | null; error: string | null }`
   - Cleans up interval on unmount
2. Create `src/contexts/HealthContext.tsx`:
   - Wraps the app to provide health status globally
   - `useIsOnline()` hook returns `boolean` — true when status is `"connected"`
   - Feature views can use `useIsOnline()` to disable action buttons when offline
3. Create `src/components/ConnectionIndicator.tsx`:
   - Small component for the header: colored dot + status text
   - Uses Tailwind: green-500, amber-500, red-500 for the dot colors
   - Tooltip showing last check time and Control Service URL
4. Create `src/components/DisconnectedBanner.tsx`:
   - Full-width warning banner below the header
   - Shows the configured `VITE_API_BASE_URL` in the error message
   - Dismissible (X button), but reappears if still disconnected on next poll
5. Wire `HealthProvider` into `src/App.tsx` (wrapping the layout)
6. Update `Header.tsx` to include `ConnectionIndicator`
7. Update `DashboardLayout.tsx` to include `DisconnectedBanner` below the header when disconnected
8. Configure production build:
   - Vite's default `npm run build` already produces optimized output with hashed assets
   - Add `public/_redirects` with `/* /index.html 200` for Netlify SPA support
   - Add `public/nginx.conf.example` with `try_files $uri /index.html` for nginx deployments
9. Run build and document output size in this task's status updates

### Dependencies

- SMET-T-0205 (Project Scaffold) — Vite build system
- SMET-T-0206 (Application Shell) — Header and DashboardLayout components to wire into
- SMET-T-0208 (API Client Layer) — `checkHealth()` function and `HealthStatus` type

### Risk Considerations

- If the Control Service is not running during development, the health check will always show "Disconnected". This is expected and correct behavior. Developers should start the Control Service for a full integration test.
- Polling interval (30s) is a balance between responsiveness and not hammering the server. Can be tuned later if needed.

### File Structure Created

```
src/
  hooks/
    useHealthCheck.ts
  contexts/
    HealthContext.tsx
  components/
    ConnectionIndicator.tsx
    DisconnectedBanner.tsx
public/
  _redirects
  nginx.conf.example
```

## Status Updates

*To be added during implementation*