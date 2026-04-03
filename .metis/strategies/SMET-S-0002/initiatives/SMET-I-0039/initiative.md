---
id: machine-connectivity-and-trust
level: initiative
title: "Machine Connectivity and Trust"
short_code: "SMET-I-0039"
created_at: 2026-03-17T19:56:51.580311+00:00
updated_at: 2026-03-27T21:07:51.761818+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#feature-remote-management"
  - "#category-infrastructure"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: machine-connectivity-and-trust
---

# Bridge Connectivity and Handshake Initiative

## Context

This is the foundational initiative for the Shepherd remote management system (SMET-S-0002). Before any session can be started remotely, the bridge daemon must be able to connect to the central server, identify itself, advertise available projects, and maintain a persistent connection with reconnection capability.

The detailed implementation design is in `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`. This initiative covers the bridge daemon lifecycle, the `bridge.hello`/`bridge.welcome` handshake, project discovery, and the WebSocket connection management. This is **MVP initiative 1 of 3**.

**Pre-requisite**: None — Shepherd is a separate repository from ultra-metis.

**Components touched**: Bridge daemon (`bridge/` crate — daemon lifecycle, config, WebSocket client, project discovery), Central Server (`server/` crate — WebSocket hub, machine registry), Protocol (`protocol/` crate — bridge handshake messages, envelope types).

## Goals & Non-Goals

**Goals:**
- Protocol crate with message envelope, bridge handshake types (`bridge.hello`, `bridge.welcome`), and serde serialization
- Bridge daemon that starts via `shepherd-bridge start`, connects outbound to server via WebSocket
- `bridge.hello` message on connect: protocol version, machine ID, machine name, bridge version, available agents, available projects
- Server responds with `bridge.welcome` (accepted/rejected, pending requests from while bridge was disconnected)
- Project discovery: bridge scans configured directories and advertises available project paths
- Bridge configuration via TOML (`~/.config/shepherd/bridge.toml`) — server URL, machine identity, project scan dirs, adapter settings
- Server configuration via TOML (`~/.config/shepherd/server.toml`) — host, port, database path, heartbeat intervals
- WebSocket reconnection with exponential backoff on disconnect
- `session.sync` message after reconnection to reconcile state between bridge and server
- Server tracks bridge online/offline/stale status from WebSocket connection liveness
- Bridge CLI: `start`, `stop`, `status`, `config` subcommands
- SQLite persistence on server for machine registry (known machines, last seen)
- `user_id` column on all tables from day one (single seeded user in MVP)

**Non-Goals:**
- Session management and PTY allocation (SMET-I-0040)
- Interaction queue and web UI (SMET-I-0041)
- Machine trust tiers and explicit approval flow (post-MVP — MVP trusts all connecting bridges)
- Machine revocation (post-MVP)
- Policy enforcement (SMET-I-0044, post-MVP)
- Advanced machine health metrics (post-MVP)

## Detailed Design

See full spec: `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`

### Protocol Crate (`protocol/`)
- Message envelope: `{ type, id (uuid), timestamp (RFC3339), payload }` — one JSON message per WebSocket text frame
- `bridge.hello` message: protocol_version, machine_id, machine_name, bridge_version, available_agents, available_projects
- `bridge.welcome` message: accepted (bool), protocol_version, server_version, pending_requests (session.start requests made while bridge was disconnected)
- `session.sync` message: machine_id, active_sessions list — sent after reconnection to reconcile state
- Protocol version field for forward compatibility; server rejects incompatible versions

### Bridge Daemon (`bridge/`)
- CLI entry point with `start`, `stop`, `status`, `config`, `wrap`, `notify` subcommands
- Daemon lifecycle: start on login or manually, connect to server, discover projects, monitor sessions
- WebSocket client with exponential backoff reconnection (5s initial, 60s max)
- On connect: send `bridge.hello`, wait for `bridge.welcome`, send `session.sync` with current state
- Process `pending_requests` from welcome message (session starts requested while disconnected)
- Project discovery: scan configured directories (2 levels deep), expose path + name pairs
- Local Unix socket for `notify` subcommand (hook-detected events sent to running daemon)

### Bridge Configuration
```toml
# ~/.config/shepherd/bridge.toml
[server]
url = "ws://localhost:8420/bridge"
reconnect_interval_ms = 5000
max_reconnect_interval_ms = 60000

[machine]
id = "dans-macbook"
name = "Dan's MacBook Pro"

[projects]
scan_dirs = ["/Users/dan/Code"]

[adapters.claude-code]
binary = "claude"
default_model = "opus"
hook_pipe_dir = "/tmp/shepherd/hooks"
```

### Central Server — WebSocket Hub (`server/`)
- Accepts WebSocket upgrades at `/bridge`
- Expects `bridge.hello` as first message, responds with `bridge.welcome`
- Ping/pong keepalive every 15 seconds
- Tracks bridge connection status: connected (WebSocket open), stale (ping timeout), disconnected (WebSocket closed)
- On disconnect: marks all sessions from that bridge as `status: unknown`
- On reconnect: processes `session.sync` to reconcile, clears stale sessions

### Central Server — REST API (MVP — machine endpoints)
- `GET /api/machines` — list connected bridges/machines with status
- `GET /api/machines/:id/projects` — list available projects on a machine

### Persistence
- SQLite at `~/.config/shepherd/server.db`
- Machine registry table: id, machine_id, machine_name, bridge_version, last_seen, user_id
- In-memory: active WebSocket connections, current bridge status

## Multi-Tenancy Notes

- `user_id` column on machines table from day one — seeded to `user_id=1` in MVP
- MVP: no auth middleware, no login flow, no trust tiers — single user, all bridges trusted
- All API queries include `WHERE user_id = :current_user` pattern even in MVP (habit, not enforcement)
- Future: swap in auth middleware that resolves JWT → user_id; all downstream scoping already works

## Alternatives Considered

- **Agent-based polling (bridge polls for commands)**: simpler NAT traversal but higher latency for command delivery; rejected in favor of persistent outbound WebSocket connection
- **REST registration instead of WebSocket handshake**: simpler but loses the persistent bidirectional channel needed for session.start commands; rejected
- **mDNS/local discovery instead of central registry**: works only on local network, breaks the remote control model; rejected

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope**

Relevant ADR decision points:
- **#1 Rename**: All references to "Cadre" in this initiative become "Cadre." The Machine Runner daemon runs in a Cadre-managed project, not an cadre project. API paths, config keys, and documentation references must use the Cadre namespace.
- **#3 SDD-style execution**: The Machine Runner must support the new execution model where a session may spawn multiple fresh subagents per task (not just a single long-running process). The process supervisor in the runner needs to handle orchestrated multi-subagent sessions, not just a single AI process. Session start commands may include subagent dispatch configuration.
- **#7 SubagentStart hook**: The Machine Runner must ensure the SubagentStart hook is active in the execution environment so that any subagents spawned during a remote session receive Cadre project context. The runner's environment setup should verify hook availability.

No changes needed for: #2 (superpowers dependency is orchestration-level, not runner-level), #4 (worktree delegation is execution-level), #5 (task claiming is orthogonal to machine connectivity), #6 (architecture hooks don't affect machine registration).

## Implementation Plan

1. Create `shepherd/` repository with Cargo workspace (protocol, server, bridge crates)
2. Implement protocol crate: message envelope, bridge.hello, bridge.welcome, session.sync types with serde
3. Implement bridge daemon CLI (`start`, `stop`, `status`, `config`) with TOML config loading
4. Implement bridge WebSocket client with exponential backoff reconnection
5. Implement bridge project discovery (scan configured directories)
6. Implement server Axum setup with WebSocket endpoint at `/bridge`
7. Implement server WebSocket hub: accept connections, bridge.hello/welcome handshake, ping/pong keepalive
8. Implement server machine registry (SQLite — machine table with user_id)
9. Implement bridge local Unix socket for `notify` subcommand
10. Integration test: bridge starts → connects → hello/welcome → server shows machine online → bridge disconnects → server marks stale → bridge reconnects → session.sync reconciles