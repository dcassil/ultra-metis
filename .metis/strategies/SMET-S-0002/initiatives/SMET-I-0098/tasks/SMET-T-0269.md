---
id: auto-start-on-login-and-platform
level: task
title: "Auto-Start on Login and Platform Service Integration"
short_code: "SMET-T-0269"
created_at: 2026-03-28T16:52:41.193612+00:00
updated_at: 2026-03-28T17:44:35.612680+00:00
parent: SMET-I-0098
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0098
---

# Auto-Start on Login and Platform Service Integration

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

Implement auto-start on system login for all three platforms, plus toast notifications for key runner events (session started, approval needed, connection lost).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] **macOS**: Register as Login Item via `SMAppService` or a LaunchAgent plist at `~/Library/LaunchAgents/com.cadre.machine-runner.plist`
- [ ] **Windows**: Add entry to `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` registry key
- [ ] **Linux**: Create `.desktop` file in `~/.config/autostart/cadre-machine-runner.desktop`
- [ ] Auto-start controlled by `auto_start` setting — toggling it adds/removes the login item
- [ ] Auto-start launches with `--minimized` flag so it goes straight to tray
- [ ] Toast/system notifications via Tauri's notification plugin for: "Session started on {repo}", "Session needs approval", "Connection to server lost", "Connection restored"
- [ ] Notifications respect OS-level notification settings (user can disable in OS)
- [ ] Notification for session approval is actionable: clicking it opens the approval dialog
- [ ] Tauri IPC command: `set_auto_start(enabled: bool)` that handles platform-specific registration

## Implementation Notes

### Technical Approach
- Use `tauri-plugin-autostart` (official Tauri plugin) which handles all 3 platforms
- Notifications via `tauri-plugin-notification` (official) — supports action buttons on macOS/Windows
- Auto-start registration happens when user toggles the setting, not at install time
- The `--minimized` flag is passed as an argument in the auto-start registration

### Dependencies
- SMET-T-0265 (Tauri scaffold), SMET-T-0266 (Settings — `auto_start` flag)
- `tauri-plugin-autostart`, `tauri-plugin-notification`

## Status Updates

*To be added during implementation*