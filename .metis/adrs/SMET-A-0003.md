---
id: 001-installable-machine-runner
level: adr
title: "Installable Machine Runner: Framework, Settings, and Security Model"
number: 1
short_code: "SMET-A-0003"
created_at: 2026-03-28T16:40:25.699643+00:00
updated_at: 2026-03-28T16:51:57.649865+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: Installable Machine Runner: Framework, Settings, and Security Model

## Context

The Machine Runner (`apps/machine-runner/`) is currently a headless Rust daemon with no UI, configured via a TOML file at `~/.config/cadre/machine-runner.toml`. Users must:
1. Manually build the binary from source (or download a release)
2. Manually create/edit the TOML config
3. Manually run the daemon (no OS service integration)
4. Have no visibility into runner status without checking the Control Dashboard

For non-developer users and broader adoption, the runner needs:
- A **one-click installer** for macOS, Windows, and Linux
- A **minimal system tray UI** for status, settings, and enable/disable
- A **proper settings model** so users don't hand-edit TOML files
- **Security options** for controlling what the runner can do on the local machine

### Current Connection Model
- HTTP polling: heartbeat every 30s to `POST /api/machines/{id}/heartbeat`
- Command polling: `GET /api/machines/{id}/commands` on each heartbeat cycle
- Static Bearer token auth from config file
- No persistent connection (no WebSocket/SSE)
- Config: `control_service_url`, `machine_name`, `api_token`, `heartbeat_interval_secs`, `repo_directories`

## Decision Points (Requiring Human Input)

### Decision 1: UI Framework for Cross-Platform Installer

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **Tauri** (Rust + Web UI) | Same Rust language as runner core; small binary (~5-10MB); native installers for all 3 OS; system tray support; web-based UI means easy styling | Smaller ecosystem than Electron; webview rendering varies by OS | Low | Medium |
| **Electron** (Node.js + Web UI) | Massive ecosystem; proven for system tray apps; identical rendering across OS | Huge binary (~150MB+); different language (JS/TS) from runner core; memory-heavy | Medium | Medium |
| **Native per-platform** (Swift/WPF/.NET MAUI/GTK) | Best OS integration; smallest footprint | 3 separate codebases; massive maintenance cost | High | Very High |
| **Flutter** (Dart) | Single codebase; good desktop support; fast UI | Different language; desktop support still maturing; large binary | Medium | Medium |

**Recommendation**: **Tauri** — same Rust backend, tiny binaries, built-in installer generation (`tauri build` produces `.dmg`, `.msi`, `.deb`/`.AppImage`), native system tray via `tauri-plugin-system-tray`. The runner core can be embedded directly as a Rust library.

### Decision 2: Settings and Configuration

**Current settings** (TOML file):
- `control_service_url` — server URL
- `machine_name` — display name
- `api_token` — bearer token
- `heartbeat_interval_secs` — polling frequency
- `repo_directories` — paths to scan for repos

**Proposed additional settings for installable UI**:

| Setting | Type | Description | Default |
|---------|------|-------------|---------|
| `auto_start` | bool | Start runner on system login | true |
| `start_minimized` | bool | Start in system tray (no window) | true |
| `enabled` | bool | Master enable/disable toggle | true |
| `max_concurrent_sessions` | int | Limit simultaneous AI sessions | 1 |
| `allowed_repos` | string[] | Whitelist of repo paths (empty = use repo_directories) | [] |
| `blocked_repos` | string[] | Blacklist of repo paths | [] |
| `require_approval_for_sessions` | bool | Require local UI approval before starting any session | false |
| `log_level` | enum | Logging verbosity (error/warn/info/debug) | info |
| `update_channel` | enum | Auto-update channel (stable/beta/nightly) | stable |
| `auto_update` | bool | Enable automatic updates | true |

### Decision 3: Security Options

| Setting | Type | Description | Default |
|---------|------|-------------|---------|
| `local_approval_required` | bool | Pop up a local confirmation dialog before ANY remote session starts on this machine | false |
| `allowed_autonomy_levels` | string[] | Which autonomy levels the runner will accept (normal, stricter, autonomous) | ["normal", "stricter"] |
| `block_autonomous_mode` | bool | Hard block on `autonomous` mode regardless of server policy | true |
| `allowed_action_categories` | string[] | Local override — which action categories are permitted | all |
| `blocked_action_categories` | string[] | Local override — which categories are denied regardless of server | [] |
| `restrict_to_repos` | bool | Only allow sessions in explicitly listed repos | false |
| `session_timeout_minutes` | int | Auto-kill sessions after N minutes (0 = no limit) | 0 |
| `network_isolation` | bool | Block spawned AI processes from making network requests | false |
| `token_storage` | enum | Where to store the API token (keychain/config_file/env_var) | keychain |

### Decision 4: Token and Credential Management

| Option | Pros | Cons |
|--------|------|------|
| **OS Keychain** (macOS Keychain, Windows Credential Manager, libsecret) | Most secure; standard practice; `keyring` Rust crate | Adds OS-specific dependency |
| **Encrypted config file** | Cross-platform; no OS dependency | Need to manage encryption key; less secure than keychain |
| **Plain text in config** (current) | Simple | Insecure; token visible to any process with file access |

**Recommendation**: **OS Keychain** as default, with config file fallback for headless/CI environments.

### Decision 5: Installer Distribution

| Platform | Format | Tool |
|----------|--------|------|
| macOS | `.dmg` with drag-to-Applications | Tauri bundler |
| Windows | `.msi` installer | Tauri bundler (WiX) |
| Linux | `.deb` + `.AppImage` + `.rpm` | Tauri bundler |

Auto-update via Tauri's built-in updater (checks GitHub Releases or custom endpoint).

### Decision 6: System Tray UI Scope

Minimal tray UI with:
- **Status indicator**: green (connected), yellow (pending approval), red (disconnected), gray (disabled)
- **Menu items**: Enable/Disable toggle, Open Settings, View Active Sessions (opens dashboard URL), Quit
- **Settings window**: All settings from Decisions 2 & 3 in a form UI
- **Session notification**: Toast/notification when a new session starts or needs local approval
- **Uninstall**: Menu item that removes config, stops service, and offers to unregister from server

## Decisions Made

1. **Framework**: **Tauri** — Rust backend, tiny binaries, native installers, system tray support
2. **Settings**: As proposed, with `block_autonomous_mode` defaulting to **false** and `local_approval_required` defaulting to **false**
3. **Token storage**: **OS Keychain** as default, config file fallback for headless/CI
4. **Installer distribution**: Tauri bundler producing `.dmg`, `.msi`, `.deb`/`.AppImage` via GitHub Actions
5. **Auto-update**: Tauri updater plugin checking GitHub Releases
6. **Auto-start**: `auto_start` defaults to **true**, `start_minimized` defaults to **true** — runner launches silently on login

## Rationale

Tauri is the natural choice: same Rust language as the runner core, the runner crate can be embedded directly as a library dependency (no IPC overhead), binaries are ~5-10MB vs Electron's 150MB+, and Tauri's bundler handles cross-platform installers out of the box. The security defaults are permissive (no blocking, no local approval) because the server-side policy layer (SMET-I-0044) already handles restrictions — local settings are an opt-in additional layer for security-conscious users.

## Consequences

### Positive
- Non-developers can install and use the runner
- Security-conscious users can restrict runner capabilities locally
- System tray provides at-a-glance status
- Cross-platform from a single codebase (Tauri)
- OS keychain protects API tokens

### Negative
- Adds a Tauri/frontend build pipeline alongside the existing Rust workspace
- System tray behavior varies across Linux desktop environments
- Auto-update infrastructure needs hosting

### Neutral
- The headless daemon mode must remain available for CI/server environments
- Config file format may change from TOML to a structured settings store