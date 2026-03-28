# Cadre Machine Runner - E2E Test Procedures

## Prerequisites

- Control API running at localhost:3000
- Valid API token (cadre-mvp-static-token)

## Test 1: Fresh Install Flow

1. Delete `~/.config/cadre/settings.json` and any keychain entry for "cadre-machine-*"
2. Launch app: `npm run tauri dev`
3. Verify setup wizard appears
4. Enter server URL: `http://localhost:3000`
5. Enter token: `cadre-mvp-static-token`
6. Enter machine name: `test-machine`
7. Complete wizard
8. Verify: tray icon appears, status changes to PendingApproval or Active
9. Verify: `~/.config/cadre/settings.json` created with the entered values

## Test 2: Settings Persistence

1. Open Settings (tray -> Settings...)
2. Go to the Behavior tab, change heartbeat interval to 60
3. Click Save
4. Close and reopen the app
5. Verify heartbeat interval is still 60

## Test 3: Security Enforcement

1. Open Settings -> Security tab
2. Enable "Block Autonomous Mode"
3. Click Save
4. From the Control Dashboard, create a session with `autonomy=autonomous`
5. Verify runner rejects it (session fails with `local_policy_violation`)

## Test 4: Enable/Disable Toggle

1. Right-click tray -> Disable
2. Verify heartbeat stops (check server logs)
3. Right-click tray -> Enable (menu item text toggles)
4. Verify heartbeat resumes

## Test 5: Uninstall

1. Right-click tray -> Uninstall...
2. Verify the uninstall confirmation dialog appears in the main window
3. Verify the "Deregister from server" checkbox is checked by default
4. Click Uninstall
5. Verify: app shows spinner then exits
6. Verify: `~/.config/cadre/settings.json` is deleted
7. Verify: keychain entry for the machine is removed (use `security find-generic-password -s cadre-machine-<name>` on macOS)
8. Verify: machine shows as revoked on server (if deregister was checked)

### Test 5b: Uninstall Cancel

1. Right-click tray -> Uninstall...
2. Click Cancel
3. Verify: returns to Settings page, no data is deleted

## Test 6: Headless Mode

1. Run: `cargo run -p cadre-machine-runner`
2. Verify: runner starts from TOML/JSON config, no UI
3. Verify: heartbeats are sent, sessions can be assigned
4. Verify: all existing functionality works (`cargo test -p cadre-machine-runner`)

## Test 7: Auto-Start

1. Open Settings -> Behavior tab
2. Enable "Auto-start on login"
3. Click Save
4. Log out and log back in (or check LaunchAgents on macOS)
5. Verify: the runner app starts automatically with `--minimized` flag
6. Verify: tray icon appears, runner connects to server

## Test 8: Desktop Notifications

1. Start the runner with valid credentials
2. Disconnect the control server (stop it)
3. Verify: a "Connection Lost" notification appears
4. Restart the control server
5. Verify: a "Connected" notification appears when heartbeat reconnects
