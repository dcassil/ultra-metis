---
id: control-dashboard-session-start
level: task
title: "Control Dashboard Session Start Flow"
short_code: "SMET-T-0230"
created_at: 2026-03-27T21:00:40.318732+00:00
updated_at: 2026-03-27T23:59:18.007365+00:00
parent: SMET-I-0040
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0040
---

# Control Dashboard Session Start Flow

## Parent Initiative

[[SMET-I-0040]] — Remote Session Lifecycle

## Objective

Build the session start flow in the Control Dashboard — the multi-step form that lets users create and launch a new AI session on a registered machine.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] "Start Session" button on dashboard, navigates to `/sessions/new`
- [ ] Machine selector: dropdown of trusted, online machines. Shows name, platform, connectivity.
- [ ] Repo selector: repos available on selected machine. User picks target repo.
- [ ] Session details: title (required), instructions textarea (required), autonomy level picker (Normal/Stricter/Autonomous)
- [ ] Optional fields: work item ID input (Cadre short code), context textarea
- [ ] Submit calls `POST /api/sessions`, on success redirect to `/sessions/{id}`
- [ ] Error handling: inline error display for validation failures
- [ ] Form validation: title, instructions, machine, repo all required
- [ ] Responsive layout using existing design system components
- [ ] `sessions.ts` API module with `createSession()`, `getSession()`, `listSessions()`

## Implementation Notes

### Technical Approach
- Single page with conditional sections (no wizard for MVP)
- Reuse FormInput, Select, Button, Card components
- Autonomy level: three radio cards with descriptions
- Add route: `/sessions/new` → `NewSessionPage`

### Dependencies
- SMET-T-0225 (provides POST /api/sessions)
- Existing dashboard design system and API client

## Status Updates

*To be added during implementation*