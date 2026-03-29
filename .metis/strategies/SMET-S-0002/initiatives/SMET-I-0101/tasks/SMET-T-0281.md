---
id: persistent-machine-identity-local
level: task
title: "Persistent Machine Identity: Local ID Storage and Re-Registration Support"
short_code: "SMET-T-0281"
created_at: 2026-03-29T00:43:13.880244+00:00
updated_at: 2026-03-29T01:08:24.497614+00:00
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

# Persistent Machine Identity: Local ID Storage and Re-Registration Support

Covers Initiative Issue 5. Prevents future orphaned machines on runner restart.

## Objective

Persist the machine_id to local disk after first registration so that subsequent runner restarts re-register with the same ID instead of creating orphaned entries. Modify the server-side registration endpoint to support re-registration with an existing ID.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] After first registration, machine_id is saved to a local file (`~/.cadre/machine_id` or Tauri app data dir)
- [ ] On runner startup, if persisted machine_id file exists, it is loaded and sent in the registration request
- [ ] `RegisterRequest` includes an optional `machine_id` field
- [ ] Server-side: if `machine_id` is provided and matches an existing record with same name, updates the existing record (resets connectivity_status to online, updates last_heartbeat, updates repos)
- [ ] Server-side: if `machine_id` is provided but no matching record exists, falls through to new registration (generates new UUID)
- [ ] Server-side: if `machine_id` is provided but name doesn't match, rejects with 409 Conflict
- [ ] Runner restart no longer creates duplicate machine entries
- [ ] Existing tests still pass

## Implementation Notes

### Technical Approach

**Machine Runner — client.rs**:
1. Add `machine_id: Option<String>` field to `RegisterRequest`

**Machine Runner — runner.rs**:
1. On startup: Check for persisted ID file. If exists, load into `self.machine_id`
2. In `register()`: If `self.machine_id` is Some, include it in RegisterRequest
3. After successful registration: Save the returned `response.id` to the persistence file
4. Persistence location:
   - Tauri mode: Use `app_data_dir()` from tauri paths
   - Headless mode: Use `~/.cadre/machine_id` (create dir if needed)
   - File format: Plain text, just the UUID string

**Control API — routes.rs**:
1. Modify `register_machine` handler:
   - If `body.machine_id` is Some:
     - Query DB for that machine_id
     - If found AND name matches: Update existing record (status, heartbeat, repos), return 200 with same ID
     - If found AND name doesn't match: Return 409 Conflict
     - If not found: Fall through to normal new registration
   - If `body.machine_id` is None: Normal new registration (current behavior)

**Control API — db.rs**:
1. Add `update_machine_registration(conn, machine_id, body, now)` function
   - Updates: name, platform, capabilities, last_heartbeat, updated_at
   - Re-syncs repos (delete old, insert new)
   - Sets connectivity_status to 'online' if machine was previously offline/stale

### Files to Change
- `apps/machine-runner/src/client.rs` — RegisterRequest field
- `apps/machine-runner/src/runner.rs` — persist/load machine_id
- `apps/control-api/src/routes.rs` — re-registration logic
- `apps/control-api/src/db.rs` — update_machine_registration function

### Dependencies
None — can be done independently. Works with SMET-T-0280 (deletion) to clean up existing orphans.

## Status Updates

*To be added during implementation*