---
id: dashboard-policy-management-ui
level: task
title: "Dashboard Policy Management UI"
short_code: "SMET-T-0237"
created_at: 2026-03-28T00:15:50.710114+00:00
updated_at: 2026-03-28T00:31:07.537942+00:00
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

# Dashboard Policy Management UI

## Parent Initiative

[[SMET-I-0044]] — Policy and Safe Execution

## Objective

Build the policy management UI in the Control Dashboard — editors for machine-level and repo-level policies on the machine detail page.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Machine detail page gains a "Policy" tab/section with the policy editor
- [ ] Machine policy editor: toggle switches for each action category (allowed/blocked), max autonomy level dropdown, session mode selector (Normal/Restricted/Elevated), multi-select for require_approval_for actions
- [ ] Repo policy editor: accessible from machine detail's repo list, same controls as machine policy but scoped to one repo. Shows "Inherits from machine" for unset fields.
- [ ] Save button calls `PUT /api/machines/{id}/policy` (or repo variant), shows success/error feedback
- [ ] "Effective Policy" view: read-only merged view showing what actually applies for a given repo (calls effective policy endpoint)
- [ ] Session mode badge visible on machine list and machine detail header
- [ ] Policies page stub (`/policies`) replaced with a page listing all machines and their session modes with links to edit
- [ ] TypeScript types for policy request/response, API functions in a `policies.ts` module
- [ ] TypeScript compiles cleanly

## Implementation Notes

### Technical Approach
- Policy editor as a form component reusable for both machine and repo scope
- Action category toggles: use Toggle component with category label and description
- Effective policy displayed as a read-only summary card below the editors
- Add `policies.ts` API module with `getMachinePolicy`, `updateMachinePolicy`, `getRepoPolicy`, `updateRepoPolicy`, `getEffectivePolicy`
- Replace stub PoliciesPage with a real page

### Dependencies
- SMET-T-0234 (Policy CRUD API)
- Existing dashboard design system

## Status Updates

*To be added during implementation*