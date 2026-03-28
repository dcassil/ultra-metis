---
id: scaffold-tauri-app-and-system-tray
level: task
title: "Scaffold Tauri App and System Tray"
short_code: "SMET-T-0265"
created_at: 2026-03-28T16:52:36.642918+00:00
updated_at: 2026-03-28T16:52:36.642918+00:00
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

# Scaffold Tauri App and System Tray

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

Create the Tauri application scaffold at `apps/runner-desktop/` with system tray integration, status icons, and the basic tray menu. This is the shell that will host all UI and embed the runner core.

## Acceptance Criteria

- [ ] `apps/runner-desktop/` directory with Tauri v2 project structure: `src-tauri/` (Rust backend), `src/` (React/TS frontend), `package.json`, `index.html`
- [ ] `src-tauri/Cargo.toml` depends on `cadre-machine-runner` crate from workspace
- [ ] System tray icon with 4 state variants: green (connected), yellow (pending), red (disconnected), gray (disabled)
- [ ] Tray right-click menu: Enable/Disable toggle, Settings (opens window), View Sessions (opens dashboard URL in browser), Quit
- [ ] Left-click on tray icon toggles the settings window
- [ ] Tray icon state updates in real-time based on `RunnerHandle::status()` via a `watch` channel
- [ ] App starts with no visible window (tray-only) when `start_minimized` is true
- [ ] Tauri IPC commands exposed: `get_status`, `start_runner`, `stop_runner`, `toggle_enabled`
- [ ] React frontend scaffolded with Vite + TypeScript + Tailwind (matching dashboard patterns)
- [ ] `cargo tauri dev` launches the app successfully on macOS
- [ ] App added to workspace `Cargo.toml` members

## Implementation Notes

### Technical Approach
- Use `tauri` v2 with `tauri-plugin-shell` for opening URLs and `tray-icon` plugin for system tray
- Tray icons: create 4 SVG/PNG icons (16x16 and 32x32) for each state, or use a single icon with colored overlays
- Tauri IPC: `#[tauri::command]` functions in `commands.rs` that call `RunnerHandle` methods
- Status polling: spawn a background task that watches `RunnerHandle::status()` and updates tray icon + menu state
- Frontend: minimal React app — just a settings page for now (wizard and detail pages come in later tasks)

### Dependencies
- SMET-T-0264 (Runner Library API — provides `RunnerHandle`)
- Tauri CLI: `cargo install tauri-cli` (or use npx @tauri-apps/cli)

## Status Updates

*To be added during implementation*