---
id: remote-session-lifecycle
level: initiative
title: "Remote Session Lifecycle"
short_code: "SMET-I-0040"
created_at: 2026-03-17T19:56:52.787698+00:00
updated_at: 2026-03-28T00:15:06.081309+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#feature-remote-management"
  - "#category-autonomous-execution"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0002
initiative_id: remote-session-lifecycle
---

# Session Lifecycle and Adapter Layer Initiative

## Context

With the bridge connected to the server (SMET-I-0039), the next piece is the session model: how sessions are created, started, monitored, controlled, and terminated. This initiative also covers the core adapter abstraction — the AgentAdapter trait and the Claude Code adapter that manages PTY allocation, prompt detection, and response injection.

The detailed implementation design is in `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`. This is **MVP initiative 2 of 3**.

**Pre-requisites**: SMET-I-0039 (Bridge Connectivity and Handshake).

**Components touched**: Bridge (`bridge/` — session manager, PTY management, adapter layer, Claude Code adapter, prompt detector), Server (`server/` — session registry, session API), Protocol (`protocol/` — session lifecycle messages, interaction types).

## Goals & Non-Goals

**Goals:**
- Protocol types for session lifecycle: `session.register`, `session.deregister`, `session.heartbeat`, `session.start`, `session.start_failed`, `session.cancel`
- Protocol types for interaction: `interaction.prompt` (with prompt_type: approval/confirm/choice/freeform/notification), `interaction.response`, `interaction.timeout`
- AgentAdapter trait: `start_session`, `stop_session`, `interaction_stream`, `send_response`, `output_stream`
- Claude Code adapter implementing AgentAdapter: PTY allocation via `openpty()`, process supervision, ANSI sanitization
- Hybrid prompt detection: hooks for tool permissions + terminal pattern matching for questions/choices/confirms + idle detection fallback
- Response injection: translate InteractionResponse back to terminal input (y/n, selected key, freeform text)
- `shepherd-bridge wrap <command>` — transparent PTY wrapper for local Claude Code sessions with full interaction relay
- Session detection for locally-started sessions via Claude Code hook (`SessionStart` → `shepherd-bridge notify`)
- Session manager: start/stop/monitor sessions, handle remote-started and locally-wrapped sessions
- Server session registry: in-memory tracking of all sessions across all bridges
- Server session REST API: list, detail, launch (proxied to bridge), cancel (proxied to bridge)
- Session heartbeat (every 30s) with status: working / waiting_for_input / idle

**Non-Goals:**
- Web UI views (SMET-I-0041)
- Output streaming to web UI (post-MVP)
- Session history persistence (SMET-I-0043, post-MVP)
- Work item attachment (SMET-I-0045, post-MVP)
- Session autonomy levels (post-MVP)
- Pause/resume (post-MVP — MVP has start and stop only)

## Detailed Design

See full spec: `docs/superpowers/specs/2026-03-19-shepherd-remote-agent-management-design.md`

### Session Status Model
Statuses: `Starting | Working | WaitingForInput | Idle | Completed | Crashed`
- `Starting`: session record created, bridge spawning AI process
- `Working`: AI process active, producing output
- `WaitingForInput`: interaction prompt detected, session blocked on human response
- `Idle`: no output for extended period (may need attention)
- `Completed`: AI process exited cleanly (exit code 0)
- `Crashed`: AI process exited with error or bridge lost the process

### AgentAdapter Trait (`bridge/src/adapters/traits.rs`)
```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> &str;
    fn capabilities(&self) -> AdapterCapabilities;
    async fn start_session(&self, config: SessionConfig) -> Result<SessionHandle>;
    async fn stop_session(&self, handle: &SessionHandle) -> Result<()>;
    fn interaction_stream(&self, handle: &SessionHandle)
        -> Pin<Box<dyn Stream<Item = InteractionPrompt> + Send>>;
    async fn send_response(&self, handle: &SessionHandle, response: InteractionResponse)
        -> Result<()>;
    fn output_stream(&self, handle: &SessionHandle)
        -> Option<Pin<Box<dyn Stream<Item = OutputChunk> + Send>>>;
}
```

### Claude Code Adapter (`bridge/src/adapters/claude_code.rs`)

**PTY Management:**
- Allocates PTY pair via `nix::pty::openpty()`
- Spawns `claude` process with child side as stdin/stdout/stderr
- Reads parent side in async loop, feeds bytes to prompt detector and output stream
- Writes to parent side when injecting responses

**Prompt Detection (Hybrid Strategy):**
1. **Hook-based** for tool permissions: Claude Code hooks write structured JSON to named pipe → adapter emits `approval` prompts (highest reliability)
2. **Pattern-based** for everything else: sliding window of last 50 lines, regex patterns detect confirm/choice/freeform prompts (medium reliability)
3. **Idle detection** as safety net: 30s no output after question-like content → generic `freeform` prompt with recent context (low reliability, catches missed prompts)

**Response Injection:**
| prompt_type | injection |
|-------------|-----------|
| `approval` | Write `y\n` or `n\n` to PTY |
| `confirm` | Write `y\n` or `n\n` to PTY |
| `choice` | Write selected key + `\n` (e.g., `A\n`) |
| `freeform` | Write text + `\n` to PTY |

### `shepherd-bridge wrap` Command
Transparent wrapper for local sessions with full interaction relay:
- Allocates a bridge-managed PTY, spawns Claude Code inside it
- User sees normal terminal output (passthrough)
- Bridge simultaneously parses output and relays interactions to server
- Handles signal forwarding (SIGINT, SIGTSTP, SIGWINCH), terminal capabilities, exit code propagation

### Session Detection for Non-Wrapped Sessions
- Claude Code hook (`SessionStart`) fires `shepherd-bridge notify session-start --project $CWD --pid $PPID`
- Bridge `notify` subcommand sends message to running daemon via local Unix socket
- Daemon tracks process via `ps` — visible in dashboard but **no interaction relay** (no PTY ownership)
- Dashboard shows note suggesting `wrap` for full capability

### Server — Session Registry
In-memory `HashMap<Uuid, SessionRecord>` tracking all sessions across bridges. Each record: session_id, machine_id, agent_type, project_path, project_name, status, started_at, last_heartbeat, capabilities.

### Server — Session REST API
- `GET /api/sessions` — list all active sessions
- `GET /api/sessions/:id` — session detail
- `POST /api/sessions` — launch new session (proxied to bridge via `session.start`)
- `DELETE /api/sessions/:id` — cancel session (proxied to bridge via `session.cancel`)

## Multi-Tenancy Notes

- `user_id` column on sessions table from day one — seeded to `user_id=1` in MVP
- Session registry is in-memory for MVP (rebuilt from bridge heartbeats on server restart)
- Interaction history persisted to SQLite with `user_id` scoping
- Future: auth middleware resolves JWT → user_id; session ownership enforced at API layer

## Alternatives Considered

- **Custom Claude API agent instead of PTY wrapping**: full programmatic control but loses Claude Code features (MCP integration, hooks, context management, conversation compression). Rebuilding those features is larger than the terminal parsing challenge. Rejected.
- **Direct Claude API adapter as MVP**: would give clean interaction model but requires reimplementing all MCP tool routing. Rejected for MVP; the adapter trait allows swapping to this approach later.
- **Session state in bridge only (no central state)**: simpler but breaks multi-device monitoring and recovery after disconnection. Rejected in favor of central registry.

## Cadre ADR Alignment (SMET-A-0001)

**Recommendation: Update scope**

Relevant ADR decision points:
- **#1 Rename**: All references to "Cadre" become "Cadre." Session creation API, work item linkage, and documentation use Cadre namespace.
- **#3 SDD-style execution**: This is the most impacted initiative. The session model must account for the fact that a session may internally use SDD-style fresh-subagent-per-task dispatch (via `/cadre-execute`), not just a single long-running AI process. The session state machine needs to accommodate orchestrated execution where the top-level process is a dispatcher spawning subagents. The Machine Runner's process supervisor must handle this pattern. Session state events should distinguish orchestrator-level state from individual subagent states.
- **#5 Simple task claiming**: When multiple remote sessions target the same repo, the simple file-based task claiming mechanism (`.cadre/claims/`) prevents duplicate work. The session creation flow should check for existing claims on the target work item.
- **#7 SubagentStart hook**: Sessions started remotely must have the SubagentStart hook active so all subagents within that session inherit Cadre context. The session start flow should verify hook availability as a prerequisite.

No changes needed for: #2 (peer dependency is install-time concern), #4 (worktree delegation handled by execution layer), #6 (architecture hooks are Phase 4).

## Implementation Plan

1. Implement protocol crate: session lifecycle messages (register, deregister, heartbeat, start, start_failed, cancel) and interaction messages (prompt, response, timeout) with serde
2. Implement AgentAdapter trait and AdapterCapabilities/SessionConfig/SessionHandle types
3. Implement Claude Code adapter: PTY allocation via `nix::pty::openpty()`, process spawn, ANSI sanitizer
4. Implement prompt detector: hook-based detection (named pipe reader) + pattern-based detection (regex on sliding window) + idle detection
5. Implement response injection: translate InteractionResponse to PTY writes
6. Implement `shepherd-bridge wrap` command: PTY passthrough + interaction relay
7. Implement session detection via `notify` subcommand + local Unix socket
8. Implement bridge session manager: orchestrate adapter, emit session lifecycle messages to server
9. Implement server session registry (in-memory) and session REST API
10. Implement server interaction queue: store pending interactions, route responses to bridge
11. Integration test: `wrap` a Claude Code session → prompt detected → interaction relayed to server → response sent → session resumes
12. Integration test: remote launch via `session.start` → bridge spawns PTY → session runs → cancel via `session.cancel`