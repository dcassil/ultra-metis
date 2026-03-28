---
id: dashboard-violation-log-and
level: task
title: "Dashboard Violation Log and Session Mode Display"
short_code: "SMET-T-0238"
created_at: 2026-03-28T00:15:51.674550+00:00
updated_at: 2026-03-28T00:35:37.030933+00:00
parent: SMET-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0044
---

# Dashboard Violation Log and Session Mode Display

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Build the policy violation log view and add session mode display throughout the dashboard so users always know what restrictions apply.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Violation log page (`/violations`): table of policy violations with columns: timestamp, session title (link), machine, action, scope (machine/repo), reason
- [ ] Filterable by machine, session, date range. Paginated.
- [ ] Session detail page shows violations inline in the state timeline (as special events)
- [ ] Session detail shows active policy restrictions: a "Policy" card listing what is blocked/restricted for this session's repo
- [ ] Session mode badge (`Normal`/`Restricted`/`Elevated`) shown on: session list, session detail, session start flow (after machine selection)
- [ ] Machine list shows session mode badge next to each machine
- [ ] Session start flow warns if selected machine is in `Restricted` mode (informational, doesn't block)
- [ ] TypeScript compiles cleanly

## Implementation Notes

### Technical Approach
- Violation log page fetches from `GET /api/policy-violations`
- Session timeline integration: fetch session events + violations, merge by timestamp, render violations with a warning icon/color
- Session mode badge: new `SessionModeBadge` component similar to `SessionStateBadge`
- Add violation API functions to `policies.ts`

### Dependencies
- SMET-T-0235 (Violation Logging API), SMET-T-0237 (Policy Management UI — shares types and API module)

## Status Updates

*To be added during implementation*