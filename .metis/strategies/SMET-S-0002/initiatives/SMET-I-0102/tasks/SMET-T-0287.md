---
id: session-continuation-re-enable
level: task
title: "Session Continuation: Re-enable Input and Follow-Up Session Support"
short_code: "SMET-T-0287"
created_at: 2026-03-29T01:29:13.971929+00:00
updated_at: 2026-03-29T02:19:42.567287+00:00
parent: SMET-I-0102
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0102
---

# Session Continuation: Re-enable Input and Follow-Up Session Support

## Parent Initiative

[[SMET-I-0102]]

## Objective

When a session reaches a terminal state (completed/stopped/failed), replace the disabled input with a "Continue Session" flow that creates a new follow-up session on the same machine and repo, with context from the previous session.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Terminal sessions show a "Continue Session" button in the GuidanceInput area instead of disabled input
- [ ] Clicking "Continue Session" creates a new session via the API with the same machine_id and repo_path
- [ ] The new session's instructions include context from the previous session (title, outcome summary)
- [ ] After creation, the user is navigated to the new session's detail page
- [ ] Non-terminal sessions retain existing GuidanceInput behavior (no regression)
- [ ] Control API has a `POST /api/sessions/{id}/continue` endpoint (or reuses create with parent reference)

## Implementation Notes

### Technical Approach

**Dashboard changes** (`apps/control-dashboard/`):
1. In `SessionDetailPage.tsx`: when `isTerminal`, render a `ContinueSessionButton` instead of `GuidanceInput`
2. New component `ContinueSessionButton.tsx`: shows a text input for follow-up instructions and a "Continue" button
3. API client: add `continueSession(sessionId, instructions)` function
4. On success, navigate to the new session detail page

**API changes** (`apps/control-api/`):
1. Add `POST /api/sessions/{id}/continue` endpoint that:
   - Loads the original session
   - Creates a new session with same machine_id, repo_path, autonomy_level
   - Sets instructions to: user's follow-up text + context from original session
   - Returns the new session response
2. Or: reuse existing `POST /api/sessions` with an optional `continue_from` field

### Key Files
- `apps/control-dashboard/src/pages/SessionDetailPage.tsx`
- `apps/control-dashboard/src/components/GuidanceInput.tsx`
- `apps/control-dashboard/src/api/sessions.ts`
- `apps/control-api/src/routes.rs`

## Status Updates

*To be added during implementation*