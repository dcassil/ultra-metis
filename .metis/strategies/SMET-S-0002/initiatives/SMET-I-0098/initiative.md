---
id: installable-machine-runner-with
level: initiative
title: "Installable Machine Runner with System Tray UI"
short_code: "SMET-I-0098"
created_at: 2026-03-28T16:41:33.224702+00:00
updated_at: 2026-03-28T17:24:26.731814+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: installable-machine-runner-with
---

# Installable Machine Runner with System Tray UI Initiative

**ADR**: [[SMET-A-0003]] — Installable Machine Runner: Framework, Settings, and Security Model

## Context

The Machine Runner is currently a headless Rust daemon that requires manual compilation, config file editing, and command-line execution. This limits adoption to developers comfortable with CLI tools. For Cadre's Remote AI Operations Layer to reach a broader audience, the runner needs to be a proper installable desktop application with a minimal system tray UI.

**Pre-requisites**: SMET-I-0039 (Machine Connectivity), SMET-I-0040 (Remote Session Lifecycle), SMET-I-0044 (Policy and Safe Execution).

**Components touched**: New Tauri app wrapping the existing machine-runner crate, installer pipelines, settings UI, system tray integration.

**Current state**: The runner at `apps/machine-runner/` is a pure Rust/Tokio daemon with HTTP polling, Bearer token auth, and TOML config at `~/.config/cadre/machine-runner.toml`.

## Goals & Non-Goals

**Goals:**
- One-click installable app for macOS (.dmg), Windows (.msi), and Linux (.deb/.AppImage)
- System tray icon with status indicator (connected/pending/disconnected/disabled)
- Tray menu: Enable/Disable, Open Settings, View Sessions (opens dashboard), Quit
- Settings window for all runner configuration (no more hand-editing TOML)
- Security settings: local approval for sessions, autonomy level restrictions, repo whitelisting, session timeouts
- OS keychain integration for API token storage
- Auto-start on login with minimized-to-tray option
- Auto-update mechanism
- Uninstall option that cleans up config and offers server deregistration
- Headless mode preserved for CI/server environments

**Non-Goals:**
- Full session management UI (that's the Control Dashboard's job)
- Live output streaming in the tray app (just status and notifications)
- Mobile runner (desktop only)
- Custom branding/theming

## Detailed Design

### Architecture

The Tauri app wraps the existing `cadre-machine-runner` crate as a Rust library:

```
apps/runner-desktop/          (new Tauri app)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           (Tauri entry, embeds runner core)
│   │   ├── tray.rs           (system tray setup)
│   │   ├── settings.rs       (settings persistence)
│   │   └── commands.rs       (Tauri IPC commands)
│   ├── Cargo.toml            (depends on cadre-machine-runner)
│   └── tauri.conf.json       (app config, bundler settings)
├── src/                      (web UI for settings window)
│   ├── App.tsx
│   ├── pages/Settings.tsx
│   └── components/...
├── package.json
└── index.html
```

The runner core (`apps/machine-runner/`) is refactored to expose a library API so Tauri can embed it, start/stop it, and query its state — without running it as a separate process.

### Settings Model (from ADR SMET-A-0003)

**Connection**: control_service_url, machine_name, api_token (keychain-stored)
**Behavior**: auto_start, start_minimized, enabled, heartbeat_interval_secs, max_concurrent_sessions
**Repos**: repo_directories, allowed_repos, blocked_repos, restrict_to_repos
**Security**: local_approval_required, allowed_autonomy_levels, block_autonomous_mode, session_timeout_minutes, allowed_action_categories, blocked_action_categories
**Updates**: auto_update, update_channel
**Logging**: log_level

### System Tray

- Tray icon changes color based on state: green (active+connected), yellow (pending approval), red (error/disconnected), gray (disabled)
- Left-click: toggle settings window
- Right-click: context menu with Enable/Disable, Settings, View Sessions, Quit
- Toast notifications for: session started, session needs local approval, runner disconnected

### First-Run Setup Flow

1. App opens a setup wizard (not just settings)
2. Enter control service URL
3. Enter or paste API token (stored in OS keychain)
4. Name this machine
5. Select repo directories to expose
6. Review security defaults
7. Runner registers with server, awaits approval
8. Setup complete — app minimizes to tray

### Installer Distribution

Built by Tauri's bundler + GitHub Actions:
- macOS: `.dmg` (code-signed if we have Apple Developer cert)
- Windows: `.msi` via WiX
- Linux: `.deb`, `.AppImage`, `.rpm`

Auto-update via Tauri updater plugin checking GitHub Releases.

## Alternatives Considered

- **Electron**: Proven but 150MB+ binaries, different language from runner core. Rejected for bloat.
- **Native per-platform**: Best integration but 3x maintenance cost. Rejected.
- **Flutter desktop**: Single codebase but Dart is a different ecosystem. Rejected.
- **CLI-only with systemd/launchd**: Works for developers but not for general users. Already exists as headless mode.

## Implementation Plan

1. Refactor machine-runner crate to expose library API (start, stop, status, settings)
2. Scaffold Tauri app in `apps/runner-desktop/`
3. Implement system tray with status indicator and menu
4. Build settings window UI (React/Tailwind in Tauri webview)
5. Implement OS keychain token storage
6. Build first-run setup wizard
7. Implement auto-start on login (platform-specific)
8. Set up Tauri bundler for cross-platform installers
9. Set up auto-update via Tauri updater
10. Integration testing on macOS, Windows, Linux