---
id: end-to-end-testing-and-uninstall
level: task
title: "End-to-End Testing and Uninstall Flow"
short_code: "SMET-T-0271"
created_at: 2026-03-28T16:52:43.079053+00:00
updated_at: 2026-03-28T16:52:43.079053+00:00
parent: SMET-I-0098
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0098
---

# End-to-End Testing and Uninstall Flow

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

End-to-end testing of the full installable runner workflow plus implementing the uninstall flow that cleans up config, credentials, auto-start entries, and optionally deregisters from the server.

## Acceptance Criteria

- [ ] E2E test: Fresh install → first-run wizard → server connection → machine registered → tray shows green → session started from dashboard → runner executes it
- [ ] E2E test: Settings change (disable autonomous mode) → create autonomous session from dashboard → runner rejects locally
- [ ] E2E test: Enable/disable toggle in tray → verify runner stops heartbeat when disabled, resumes when re-enabled
- [ ] E2E test: Close and reopen app → verify settings persist and runner reconnects
- [ ] Uninstall flow (accessible from tray menu "Uninstall..."): confirmation dialog → stop runner → remove auto-start entry → delete settings file → delete keychain token → optionally call `POST /api/machines/{id}/revoke` to deregister from server → show "Uninstall complete" → app exits
- [ ] Uninstall is optional — user can also just delete the app (settings remain for re-install)
- [ ] Headless mode still works: `cadre-machine-runner` binary runs without Tauri, reads from config file, no tray/UI
- [ ] All existing machine-runner tests still pass

## Implementation Notes

### Technical Approach
- E2E tests are manual test scripts documented in the task (hard to fully automate cross-platform GUI)
- Uninstall Tauri command: `uninstall(deregister: bool)` that performs cleanup steps
- Keychain deletion via `keyring` crate's `delete_credential`
- Auto-start removal via `tauri-plugin-autostart`'s disable function
- Server deregistration: call `POST /api/machines/{id}/revoke` with stored token before deleting the token
- Headless mode: the existing `main.rs` binary remains unchanged, only the Tauri app is new

### Dependencies
- All other tasks (T-0264 through T-0270) must be complete

## Status Updates

*To be added during implementation*