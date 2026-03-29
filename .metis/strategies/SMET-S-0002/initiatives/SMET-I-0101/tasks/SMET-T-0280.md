---
id: machine-deletion-api-and-bulk
level: task
title: "Machine Deletion API and Bulk Cleanup UI"
short_code: "SMET-T-0280"
created_at: 2026-03-29T00:43:12.611661+00:00
updated_at: 2026-03-29T01:02:06.830091+00:00
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

# Machine Deletion API and Bulk Cleanup UI

Covers Initiative Issue 3. Depends on Issue 5 cleanup for orphans.

## Objective

Add a true machine deletion endpoint (`DELETE /api/machines/{id}`) to the control API that removes machine records from the database, and add deletion UI to both the machine list (bulk "Remove offline" action) and machine detail page ("Remove" button).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `DELETE /api/machines/{id}` endpoint exists and removes the machine record
- [ ] DELETE endpoint returns 409 Conflict if machine has active (non-terminal) sessions
- [ ] DELETE endpoint cascades: removes machine_repos, machine policies, and machine logs for that machine
- [ ] `deleteMachine()` function added to dashboard API client
- [ ] Machine Detail page has a "Remove" button (distinct from "Revoke")
- [ ] "Remove" shows a confirmation modal warning about permanent deletion
- [ ] MachinesPage has a "Remove all offline" button in the header area
- [ ] "Remove all offline" shows confirmation with count of machines to be removed
- [ ] After removal, machine list refreshes and removed entries are gone

## Implementation Notes

### Technical Approach

**Backend — control-api**:
1. `db.rs`: Add `delete_machine(conn, machine_id)` function that:
   - Checks for active sessions (`SELECT count(*) FROM sessions WHERE machine_id = ? AND state NOT IN ('completed', 'failed', 'stopped')`)
   - If active sessions exist, return error
   - Delete from: `machine_logs`, `machine_policies`, `repo_policies`, `machine_repos`, `machines` (in that order for FK safety)
2. `routes.rs`: Add `DELETE /api/machines/{machine_id}` handler
   - Auth check (same as other machine endpoints)
   - Call `delete_machine()`, return 204 No Content on success, 409 on conflict
3. Add `DELETE /api/machines/offline` bulk endpoint that deletes all machines with connectivity_status = 'offline' and no active sessions

**Frontend — control-dashboard**:
1. `api/machines.ts`: Add `deleteMachine(id: string)` and `deleteOfflineMachines()` functions
2. `MachineDetailPage.tsx`: Add "Remove" button next to "Revoke" — red variant, with confirmation modal
3. `MachinesPage.tsx`: Add "Remove all offline" button in header, with confirmation showing count

### Files to Change
- `apps/control-api/src/db.rs`
- `apps/control-api/src/routes.rs`
- `apps/control-dashboard/src/api/machines.ts`
- `apps/control-dashboard/src/pages/MachineDetailPage.tsx`
- `apps/control-dashboard/src/pages/MachinesPage.tsx`

### Dependencies
- SMET-T-0277 (clickable machines) for reaching the detail page
- Works alongside SMET-T-0281 (persistent machine ID) to prevent future orphans

## Status Updates

*To be added during implementation*