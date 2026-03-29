---
id: session-event-history-hydration
level: task
title: "Session Event History Hydration: Persist Live Output Across Page Reloads"
short_code: "SMET-T-0282"
created_at: 2026-03-29T00:43:15.302009+00:00
updated_at: 2026-03-29T01:11:05.874284+00:00
parent: SMET-I-0101
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0101
---

# Session Event History Hydration: Persist Live Output Across Page Reloads

Covers Initiative Issue 6. Session output events are already persisted in the DB but not loaded on page mount.

## Objective

When the SessionDetailPage loads (or reloads), fetch historical session output events from the database and display them immediately in LiveOutput. Then connect the SSE stream and append only new events, providing seamless continuity across page reloads.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] On SessionDetailPage mount, historical events are fetched from `GET /api/sessions/{id}/events`
- [ ] Historical events are displayed in LiveOutput immediately (before SSE connects)
- [ ] SSE stream connects after historical load and only appends events with sequence_num > last historical event
- [ ] No duplicate events displayed (deduplication by sequence_num)
- [ ] For terminal sessions (completed/failed/stopped), only historical events are shown (no SSE connection)
- [ ] Page reload on an active session shows full conversation history + live tail
- [ ] Performance: Historical load doesn't block SSE connection (load in parallel, merge)

## Implementation Notes

### Technical Approach

**useSessionEventStream.ts** — Enhance the hook:
1. Add optional params: `initialEvents?: SessionEvent[]` and `startAfterSequence?: number`
2. On initialization, seed the events array with `initialEvents` if provided
3. When SSE events arrive, check `sequence_num > startAfterSequence` before appending
4. This prevents duplicates when historical and SSE overlap

**SessionDetailPage.tsx** — Hydrate on mount:
1. Add a `useEffect` that fetches `getSessionEvents(id)` on mount
2. Extract the max `sequence_num` from the historical events
3. Pass both to `useSessionEventStream(id, { initialEvents, startAfterSequence })`
4. For terminal sessions, skip the SSE hook entirely (just show historical)

**api/events.ts** — Verify pagination support:
1. The `GET /api/sessions/{id}/events` endpoint should return events ordered by `sequence_num`
2. If the endpoint doesn't support pagination, it's acceptable for now (events are small text records)
3. If pagination is needed later, add `?after_sequence=N&limit=500` params

### Files to Change
- `apps/control-dashboard/src/hooks/useSessionEventStream.ts`
- `apps/control-dashboard/src/pages/SessionDetailPage.tsx`
- `apps/control-dashboard/src/api/events.ts` (verify/enhance)

### Dependencies
None — independent of other tasks.

## Status Updates

*To be added during implementation*