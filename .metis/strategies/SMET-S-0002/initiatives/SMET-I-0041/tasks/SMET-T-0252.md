---
id: dashboard-approval-response-and
level: task
title: "Dashboard Approval Response and Guidance Injection UI"
short_code: "SMET-T-0252"
created_at: 2026-03-28T00:37:01.343053+00:00
updated_at: 2026-03-28T01:08:45.098269+00:00
parent: SMET-I-0041
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0041
---

# Dashboard Approval Response and Guidance Injection UI

## Parent Initiative

[[SMET-I-0041]] — Live Monitoring and Intervention

## Objective

Build the dashboard UI for responding to approval requests and injecting guidance into running sessions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] "Pending Approvals" section on session detail: shows a card for each pending approval with question text, option buttons, and optional note field
- [ ] Clicking an option calls `POST /api/sessions/{id}/respond` with the choice and optional note
- [ ] After responding, approval card transitions to "responded" state showing the chosen option
- [ ] Guidance injection UI: text input with injection type selector (Normal/Side Note/Interrupt) and Send button
- [ ] Inject calls `POST /api/sessions/{id}/inject` with message and type
- [ ] Injection history shown inline in the event stream (distinct styling — e.g., purple/italic with "You:" prefix)
- [ ] Interrupt injection requires confirmation modal (destructive action)
- [ ] Approval and injection actions disabled when session is in terminal state
- [ ] Approval notification badge on session list row when session has pending approvals
- [ ] `api/interventions.ts` module with `respondToApproval()`, `injectGuidance()`, `listPendingApprovals()`
- [ ] TypeScript compiles cleanly

## Implementation Notes

### Technical Approach
- Approval card component: renders question, options as buttons, note field as collapsible
- Guidance injection: fixed input bar at bottom of session detail (like a chat input)
- Injection types as a small dropdown/segmented control next to the send button
- Pending approvals fetched via `GET /api/sessions/{id}/approvals`, auto-refreshed via SSE events

### Dependencies
- SMET-T-0249 (Intervention API), SMET-T-0251 (Live Monitoring UI — shares session detail page)

## Status Updates

*To be added during implementation*