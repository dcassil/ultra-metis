---
id: dashboard-pending-machine-approval
level: task
title: "Dashboard Pending Machine Approval Flow"
short_code: "SMET-T-0202"
created_at: 2026-03-27T16:18:44.228942+00:00
updated_at: 2026-03-27T16:18:44.228942+00:00
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

# Dashboard Pending Machine Approval Flow

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Build the pending machine approval queue in the Control Dashboard. When a new Machine Runner registers with the Control Service, it starts in `pending` status. This task implements the UI flow for the user to review pending machines and either approve or reject them, which is the core of the trust model.

## Acceptance Criteria

- [ ] Pending machines section/banner visible on the machine list page when there are machines in `pending` status — shows a count badge (e.g., "3 machines awaiting approval")
- [ ] Pending machines queue page (or inline section) lists all `pending` machines with: name, platform, registration time, list of advertised repos
- [ ] Each pending machine has an "Approve" button and a "Reject" button
- [ ] Approve button calls `POST /machines/{id}/approve` and on success, the machine moves from the pending queue to the trusted machines list. UI updates immediately (optimistic update or refetch).
- [ ] Reject button calls `POST /machines/{id}/revoke` (revoke a pending machine effectively rejects it) with a confirmation dialog ("Are you sure you want to reject this machine?")
- [ ] After approval, the machine appears in the main machine list with trust tier "trusted" and status reflecting its heartbeat
- [ ] After rejection, the machine disappears from the pending queue (it is revoked and the runner will receive 401 on its next heartbeat)
- [ ] Approve action optionally allows setting the trust tier: "trusted" (default) or "restricted" — shown as a dropdown or radio button next to the approve button
- [ ] Toast/notification shown on successful approve or reject action
- [ ] If the approve API returns 409 (machine already approved or revoked), show an informative error message and refresh the list

## Implementation Notes

### Technical Approach
- The pending queue can be derived from the existing `GET /machines` response by filtering for `status === "pending"` — no additional API endpoint needed
- Alternatively, add a query parameter `GET /machines?status=pending` for efficiency (optional optimization)
- The approval flow is: user clicks Approve -> (optional) selects trust tier -> confirm -> `POST /machines/{id}/approve` with optional `trust_tier` body field
- The rejection flow is: user clicks Reject -> confirmation dialog -> `POST /machines/{id}/revoke`
- Consider showing the pending count as a badge in the sidebar navigation next to "Machines" to draw attention

### UI Components to Create
1. `PendingMachinesBanner` — shown at top of machine list when pending count > 0
2. `PendingMachineCard` — displays a single pending machine with approve/reject actions
3. `ApprovalDialog` — optional trust tier selection before confirming approval
4. `RejectionConfirmDialog` — "Are you sure?" confirmation before rejecting

### Dependencies
- SMET-T-0201 (Control Dashboard Machine List and Detail Views) — this builds on the machine list page
- SMET-T-0198 (Control Service Machine Registry API) — the approve and revoke endpoints must be functional

### Risk Considerations
- Race condition: if two users (future multi-user scenario) try to approve/reject the same machine simultaneously, the API should handle 409 gracefully and the UI should show a clear message
- The pending queue should auto-refresh (same polling interval as the machine list) so new registrations appear without manual refresh

## Status Updates

*To be added during implementation*