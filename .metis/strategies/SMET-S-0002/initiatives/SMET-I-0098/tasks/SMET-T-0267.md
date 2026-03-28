---
id: settings-window-and-first-run
level: task
title: "Settings Window and First-Run Setup Wizard UI"
short_code: "SMET-T-0267"
created_at: 2026-03-28T16:52:38.931013+00:00
updated_at: 2026-03-28T16:52:38.931013+00:00
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

# Settings Window and First-Run Setup Wizard UI

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

Build the React settings window with all configuration options organized into sections, and the first-run setup wizard that guides new users through initial configuration.

## Acceptance Criteria

- [ ] Settings window with tabbed/sectioned layout: Connection, Behavior, Repositories, Security, Updates
- [ ] **Connection tab**: Server URL input, machine name input, API token field (masked, with show/hide toggle), "Test Connection" button that verifies the server is reachable
- [ ] **Behavior tab**: Auto-start toggle, start minimized toggle, enabled toggle, heartbeat interval slider (15-120s), max concurrent sessions number input
- [ ] **Repositories tab**: List of repo directories with add/remove, allowed repos list, blocked repos list, restrict-to-repos toggle
- [ ] **Security tab**: Local approval required toggle, autonomy level checkboxes, block autonomous mode toggle, session timeout input, action category allow/block toggles
- [ ] **Updates tab**: Auto-update toggle, update channel selector (stable/beta), current version display, "Check for Updates" button
- [ ] Save button persists all settings and applies to running runner
- [ ] First-run wizard (shown on first launch when no settings exist): 5 steps — Welcome, Server Connection, Machine Identity, Repository Selection, Security Review → Done
- [ ] Wizard validates each step before allowing Next (e.g., tests connection before proceeding)
- [ ] Wizard stores settings and starts the runner on completion
- [ ] TypeScript compiles cleanly

## Implementation Notes

### Technical Approach
- Settings UI calls Tauri IPC commands (`get_settings`, `save_settings`, `get_token`, `set_token`)
- Wizard is a multi-step form with state tracked in React (useState for current step and accumulated data)
- "Test Connection" calls a Tauri command that does a health check GET to the server URL
- Repo directory selection: use Tauri's file dialog API (`@tauri-apps/plugin-dialog`) for native folder picker
- Reuse Tailwind + the component patterns from the Control Dashboard

### Dependencies
- SMET-T-0265 (Tauri scaffold — provides the app shell)
- SMET-T-0266 (Settings persistence — provides IPC commands)

## Status Updates

*To be added during implementation*