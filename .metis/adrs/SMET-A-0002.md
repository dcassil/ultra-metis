---
id: 001-websocket-based-machine
level: adr
title: "WebSocket-Based Machine Connectivity: Replace Polling with Persistent Connections"
number: 1
short_code: "SMET-A-0002"
created_at: 2026-03-27T23:55:27.823219+00:00
updated_at: 2026-03-27T23:55:27.823219+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/draft"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: WebSocket-Based Machine Connectivity: Replace Polling with Persistent Connections

## Context

The current machine connectivity model uses HTTP polling for both the Machine Runner → Control Service heartbeat (every 20s) and the Dashboard → Control Service status refresh (every 10s). This creates several problems:

1. **Latency in state changes**: When a machine goes offline or is revoked, it takes up to 20s for the runner to detect it, and up to 10s for the dashboard to reflect it. Status transitions (online → stale → offline) are computed from heartbeat recency with generous thresholds (2min/10min) rather than being detected in real time.

2. **Unnecessary load**: Polling generates constant HTTP traffic even when nothing changes. Each dashboard tab polls independently. With N machines and M dashboard users, the API handles `N/20 + M/10` requests per second just for liveness — scaling linearly with both.

3. **No server-push capability**: The Control Service cannot proactively notify the runner of revocation, policy changes, or session commands. The runner must discover these on its next heartbeat. This adds latency to every future initiative (sessions, live monitoring, intervention) that needs responsive communication.

4. **Dashboard UX gaps**: Users see stale data between polls. Status badge transitions are jumpy rather than smooth. There's no way to show real-time session output or live monitoring data without sub-second polling, which would be prohibitively expensive.

### Current Architecture
```
Runner --[POST /heartbeat every 20s]--> Control API
Dashboard --[GET /machines every 10s]--> Control API
```

## Decision

**Status: Under Discussion** — no decision made yet. This ADR explores the options.

Replace HTTP polling with persistent WebSocket connections for both the Machine Runner ↔ Control Service and Dashboard ↔ Control Service communication paths. HTTP REST endpoints remain for CRUD operations; WebSockets handle real-time state synchronization and push notifications.

### Proposed Architecture
```
Runner --[WebSocket: heartbeat, commands, status]--> Control API
Dashboard --[WebSocket: live status, events, output]--> Control API
Dashboard --[HTTP REST: CRUD operations]--> Control API
```

## Alternatives Analysis

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **A: WebSocket (bidirectional)** | Real-time push in both directions. Runner can receive commands instantly. Dashboard gets live updates. Industry standard. | More complex server (connection management, reconnection). Proxy/firewall compatibility. State management for connections. | Medium | M — new connection layer, but well-understood tech |
| **B: Server-Sent Events (SSE)** | Simpler than WS (HTTP-based, auto-reconnect). Good for server→client push. Works through most proxies. | Unidirectional — runner still needs HTTP POST for heartbeat. No binary frames. Less ecosystem support in Rust. | Low | S — simpler, but only solves half the problem |
| **C: HTTP long-polling** | Minimal change from current architecture. Works everywhere. | Still polling semantics. Higher latency than WS/SSE. More complex client logic. Not a real improvement for runner→service path. | Low | S — but delivers least value |
| **D: Hybrid — WS for runner, SSE for dashboard** | Each path gets the right tool. Runner needs bidirectional (commands). Dashboard only needs push. SSE simpler for browser. | Two different real-time protocols to maintain. More operational complexity. | Medium | L — two subsystems |
| **E: Keep polling, tighten thresholds** | Zero new infrastructure. Just tune 20s→5s heartbeat, 10s→2s dashboard poll. | Scales poorly. Doesn't enable push. Doesn't solve command delivery latency. Band-aid. | Low | XS — but doesn't solve the core problem |

## Key Design Considerations

### Runner ↔ Control Service (WebSocket)
- **Heartbeat**: WS ping/pong frames replace HTTP heartbeat entirely — built into the protocol
- **Connectivity detection**: Server detects disconnect within seconds via TCP keepalive/ping timeout, not computed thresholds
- **Command delivery**: Server pushes session start/stop/intervene commands to runner instantly
- **Reconnection**: Runner reconnects with exponential backoff on disconnect (reuse existing backoff logic)
- **Auth**: Initial WS handshake includes Bearer token, same as current HTTP auth
- **Multiplexing**: Single connection carries heartbeat, commands, status updates, and future session output

### Dashboard ↔ Control Service (WebSocket or SSE)
- **Live status**: Machine status changes pushed immediately (online/offline/revoked)
- **Session events**: Future session output streamed in real-time
- **Scoping**: Each WS connection authenticated and scoped to user_id — only receives events for own machines
- **Fallback**: HTTP polling remains as fallback if WS connection fails

### Migration Path
1. Add WS endpoint alongside existing REST endpoints (non-breaking)
2. Runner connects via WS, falls back to HTTP polling if WS fails
3. Dashboard subscribes to WS events, falls back to polling
4. Once stable, deprecate polling heartbeat endpoint
5. Tighten connectivity thresholds since detection is now real-time

### Rust Ecosystem
- **axum**: Native WebSocket support via `axum::extract::ws::WebSocket`
- **tokio-tungstenite**: Mature WS client for the runner
- **Dashboard**: Browser native `WebSocket` API, no additional dependencies

## Rationale

Pending decision. The recommendation leans toward **Option A (full WebSocket)** because:
- Both runner and dashboard benefit from bidirectional communication
- The runner *must* receive commands for session management (SMET-I-0040), making unidirectional SSE insufficient
- Axum has first-class WS support, minimizing new dependencies
- The migration is incremental — existing HTTP endpoints stay during transition

## Consequences

### Positive
- Real-time connectivity detection (seconds, not minutes)
- Instant command delivery to runners (critical for sessions, intervention)
- Live dashboard updates without polling overhead
- Enables streaming session output (SMET-I-0041)
- Reduces steady-state API load significantly
- Foundation for all real-time features in the Remote Operations roadmap

### Negative
- Connection lifecycle management complexity (reconnection, cleanup, connection pooling)
- More server memory per connection (one WS per runner + per dashboard client)
- Harder to debug than stateless HTTP (need WS-aware tooling)
- Some corporate networks block WebSocket — need HTTP fallback
- Integration tests more complex (need to test WS flows)

### Neutral
- HTTP REST endpoints remain for CRUD — this adds a communication layer, not replaces one
- Existing heartbeat thresholds become irrelevant (connectivity is live)
- Monitoring shifts from "check heartbeat recency" to "check connection state"

## Affected Initiatives
- **SMET-I-0040** (Remote Session Lifecycle) — sessions need instant command delivery
- **SMET-I-0041** (Live Monitoring and Intervention) — needs real-time streaming
- **SMET-I-0042** (Notifications and Mobile Control) — needs push
- **SMET-I-0046** (Operational Reliability) — connection management is a reliability concern