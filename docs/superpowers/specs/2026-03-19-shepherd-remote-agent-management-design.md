# Shepherd: Remote AI Agent Management System

**Date**: 2026-03-19
**Status**: Design

## Problem

When running multiple Claude Code CLI sessions across different terminal windows on a local machine, each session blocks when it needs human input — tool permission prompts, clarifying questions, phase transition approvals, multiple choice decisions, or freeform responses. If you're not at your desk, every session sits idle. There's no way to manage these sessions remotely or respond to them from a phone.

## Goals

1. **Unblock AI agents from anywhere** — respond to any pending interaction from a mobile-friendly web UI
2. **Session oversight** — see all active AI sessions, their status, and what they're working on
3. **Remote session launch** — start new AI sessions on the local machine from the web UI
4. **Local session registration** — sessions started in a terminal automatically appear in the UI
5. **Extensible plugin protocol** — any AI tool could implement the protocol, not just Claude Code

## Non-Goals

- Replacing Claude Code or building a custom AI agent
- Multi-tenant / multi-user access control (MVP is single-user)
- Cloud hosting (MVP runs on local network)
- Modifying Claude Code's source code

## Architecture Overview

Three components connected by a plugin protocol:

1. **Central Server + Web UI** — the hub, accessible from any device on the network
2. **Agent Bridge** — local daemon on the dev machine, manages AI sessions
3. **Plugin Protocol** — the contract between bridge and server, defining session lifecycle and interaction types

Communication direction: the bridge connects **outbound** to the server via WebSocket. This works through firewalls/NAT and scales to multi-machine setups in the future.

```
┌──────────────────────────┐
│   Phone / Browser        │
│   (Mobile-First Web UI)  │
│                          │
│   - Interaction Queue    │
│   - Session Dashboard    │
│   - Session Launcher     │
└───────────┬──────────────┘
            │ HTTPS + WSS
            ▼
┌──────────────────────────┐
│   Central Server         │
│                          │
│   - WebSocket Hub        │
│   - REST API (web UI)    │
│   - Session Registry     │
│   - Interaction Queue    │
└───────────┬──────────────┘
            │ WSS (outbound from bridge)
            ▼
┌──────────────────────────┐
│   Agent Bridge (Daemon)  │
│                          │
│   - WS Client            │
│   - Session Manager      │
│   - Adapter Layer        │
│     └─ Claude Code       │
│        Adapter (MVP)     │
│   - Managed PTY Sessions │
│     [claude ...]         │
│     [claude ...]         │
└──────────────────────────┘
```

## Design Decision: Terminal Proxy over Custom Agent

**Chosen**: Wrap Claude Code in managed pseudo-terminals and detect/relay interactions via a hybrid of Claude Code hooks and terminal pattern matching.

**Rejected alternative — Custom Agent via Claude API**: Would give full programmatic control but loses all Claude Code features (MCP integration, hooks, context management, `/` commands, conversation compression). Rebuilding those features is a larger effort than the terminal parsing challenge.

**Rejected alternative — Claude Code remote mode**: Doesn't exist. If Anthropic ships a headless/API mode for Claude Code in the future, the adapter can swap to use it while the rest of the system stays the same.

**Risk**: Terminal parsing is inherently heuristic-based. The hybrid approach (hooks for tool permissions + pattern matching for the rest) mitigates this. Worst case for an undetected prompt: the session appears idle in the UI with a "session may need attention" fallback after a timeout.

## Plugin Protocol

The protocol defines all messages exchanged between the bridge and the server over WebSocket. Messages are JSON — one JSON message per WebSocket text frame.

### Protocol Version

The current protocol version is `1`. The `bridge.hello` message includes a `protocol_version` field. The server rejects connections with incompatible protocol versions via `bridge.welcome` with an `error` field.

### Message Envelope

Every message has a common envelope:

```json
{
  "type": "string",
  "id": "uuid",
  "timestamp": "RFC3339",
  "payload": { }
}
```

The `type` field determines the payload schema. The `id` is unique per message and used for correlation (responses reference the prompt's `id`).

### Session Lifecycle Messages

#### `session.register`

Sent by the bridge when a new AI session starts (either launched remotely or detected locally).

```json
{
  "type": "session.register",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:30:00Z",
  "payload": {
    "session_id": "session-uuid",
    "agent_type": "claude-code",
    "project_path": "/Users/dan/Code/ultra-metis",
    "project_name": "ultra-metis",
    "machine_id": "dans-macbook",
    "started_at": "2026-03-19T14:30:00Z",
    "initial_prompt": "Work on SMET-T-0130",
    "capabilities": {
      "interactions": ["approval", "confirm", "choice", "freeform", "notification"],
      "features": ["metis", "git", "file-editing", "web-search"],
      "can_cancel": true,
      "supports_output_streaming": true
    }
  }
}
```

#### `session.deregister`

Sent when a session ends (process exit, user cancellation, or crash).

```json
{
  "type": "session.deregister",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T15:00:00Z",
  "payload": {
    "session_id": "session-uuid",
    "reason": "completed | cancelled | crashed | timeout",
    "exit_code": 0
  }
}
```

#### `session.heartbeat`

Sent periodically (every 30s) to indicate the session is alive and report status.

```json
{
  "type": "session.heartbeat",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:31:00Z",
  "payload": {
    "session_id": "session-uuid",
    "status": "working | waiting_for_input | idle",
    "last_activity_at": "2026-03-19T14:30:45Z"
  }
}
```

#### `session.start` (server → bridge)

Sent by the server when the user launches a session from the web UI.

```json
{
  "type": "session.start",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:30:00Z",
  "payload": {
    "request_id": "req-uuid",
    "agent_type": "claude-code",
    "project_path": "/Users/dan/Code/ultra-metis",
    "initial_prompt": "Work on SMET-T-0130",
    "options": {
      "model": "opus",
      "verbose": false
    }
  }
}
```

The bridge responds with a `session.register` on success or a `session.start_failed` on error.

#### `session.start_failed` (bridge → server)

```json
{
  "type": "session.start_failed",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:30:01Z",
  "payload": {
    "request_id": "req-uuid",
    "error": "Project path does not exist"
  }
}
```

#### `session.cancel` (server → bridge)

Sent when the user cancels a session from the web UI. The bridge should gracefully stop the session (SIGTERM, wait, SIGKILL).

```json
{
  "type": "session.cancel",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:45:00Z",
  "payload": {
    "session_id": "session-uuid",
    "reason": "user_cancelled"
  }
}
```

The bridge responds with a `session.deregister` once the session has stopped.

#### `session.sync` (bridge → server)

Sent after reconnection to reconcile state. Contains all currently running sessions. The server uses this to clear stale sessions and update its registry.

```json
{
  "type": "session.sync",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:29:05Z",
  "payload": {
    "machine_id": "dans-macbook",
    "active_sessions": [
      {
        "session_id": "session-uuid-1",
        "agent_type": "claude-code",
        "project_path": "/Users/dan/Code/ultra-metis",
        "project_name": "ultra-metis",
        "status": "working",
        "started_at": "2026-03-19T14:00:00Z",
        "capabilities": {
          "interactions": ["approval", "confirm", "choice", "freeform", "notification"],
          "features": ["metis", "git", "file-editing"],
          "can_cancel": true,
          "supports_output_streaming": true
        }
      }
    ]
  }
}
```

### Interaction Messages

#### `interaction.prompt` (bridge → server)

The AI session needs human input. The `prompt_type` determines how the web UI renders it.

**Approval** (tool permission prompts):

```json
{
  "type": "interaction.prompt",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:32:00Z",
  "payload": {
    "interaction_id": "int-uuid",
    "session_id": "session-uuid",
    "prompt_type": "approval",
    "title": "Allow Bash: git push origin main",
    "context": "Claude wants to push 3 commits to origin/main",
    "details": "Command: git push origin main\nIn: /Users/dan/Code/ultra-metis",
    "timeout_seconds": null
  }
}
```

**Confirm** (phase transitions, destructive operations):

```json
{
  "type": "interaction.prompt",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:33:00Z",
  "payload": {
    "interaction_id": "int-uuid",
    "session_id": "session-uuid",
    "prompt_type": "confirm",
    "title": "Transition SMET-I-0050 to active?",
    "context": "Initiative: Benchmark Framework Core Infrastructure\nCurrent phase: decompose\nAll 10 tasks created and reviewed.\n\nClaude says: 'All tasks are decomposed and ready. Should I transition the initiative to active and begin execution?'",
    "timeout_seconds": null
  }
}
```

**Choice** (multiple choice questions):

```json
{
  "type": "interaction.prompt",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:34:00Z",
  "payload": {
    "interaction_id": "int-uuid",
    "session_id": "session-uuid",
    "prompt_type": "choice",
    "title": "Which authentication approach?",
    "context": "Designing the API auth layer for the server component.",
    "options": [
      {
        "key": "A",
        "label": "JWT with refresh tokens",
        "description": "Stateless, scalable, but requires token rotation logic"
      },
      {
        "key": "B",
        "label": "Session cookies",
        "description": "Simpler server-side, but requires session store"
      },
      {
        "key": "C",
        "label": "API keys",
        "description": "Simplest for MVP, single-user, no expiry management"
      }
    ],
    "timeout_seconds": null
  }
}
```

**Freeform** (open-ended questions):

```json
{
  "type": "interaction.prompt",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:35:00Z",
  "payload": {
    "interaction_id": "int-uuid",
    "session_id": "session-uuid",
    "prompt_type": "freeform",
    "title": "What should the error message say when auth fails?",
    "context": "Implementing the login endpoint. Need to decide on the user-facing error message for invalid credentials.",
    "timeout_seconds": null
  }
}
```

**Notification** (no response expected):

```json
{
  "type": "interaction.prompt",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:36:00Z",
  "payload": {
    "interaction_id": "int-uuid",
    "session_id": "session-uuid",
    "prompt_type": "notification",
    "title": "Task SMET-T-0130 completed",
    "context": "All acceptance criteria met. 4 files created, 12 tests passing.",
    "timeout_seconds": null
  }
}
```

#### `interaction.response` (server → bridge)

Human sends their answer. Shape of `value` depends on `prompt_type`:

```json
{
  "type": "interaction.response",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:32:15Z",
  "payload": {
    "interaction_id": "int-uuid",
    "session_id": "session-uuid",
    "value": { "approved": true }
  }
}
```

Value shapes by prompt type:

| prompt_type | value shape |
|-------------|-------------|
| `approval` | `{ "approved": true \| false }` |
| `confirm` | `{ "approved": true \| false, "comment": "optional string" }` |
| `choice` | `{ "selected": "A" }` |
| `freeform` | `{ "text": "user's typed response" }` |
| `notification` | No response sent |

#### `interaction.timeout` (server → bridge)

Sent by the server if a prompt expires without a human response (when `timeout_seconds` is set).

```json
{
  "type": "interaction.timeout",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:37:00Z",
  "payload": {
    "interaction_id": "int-uuid"
  }
}
```

### Output Streaming Messages (Post-MVP)

These messages are defined in the protocol for forward compatibility but are **not implemented in the MVP**. The MVP Session Detail view shows session status and pending interactions only, not live terminal output.

#### `output.chunk` (bridge → server)

Streaming terminal output, sanitized of ANSI escape codes.

```json
{
  "type": "output.chunk",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:30:05Z",
  "payload": {
    "session_id": "session-uuid",
    "content": "Reading file src/main.rs...\n",
    "chunk_index": 42
  }
}
```

#### `output.milestone` (bridge → server)

Significant progress markers, extracted from output patterns.

```json
{
  "type": "output.milestone",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:31:00Z",
  "payload": {
    "session_id": "session-uuid",
    "summary": "Created 5 files, modified 2",
    "milestone_type": "progress | step_done | error"
  }
}
```

### Connection Management

#### `bridge.hello` (bridge → server)

First message after WebSocket connection. Identifies the machine.

```json
{
  "type": "bridge.hello",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:29:00Z",
  "payload": {
    "protocol_version": 1,
    "machine_id": "dans-macbook",
    "machine_name": "Dan's MacBook Pro",
    "bridge_version": "0.1.0",
    "available_agents": ["claude-code"],
    "available_projects": [
      {
        "path": "/Users/dan/Code/ultra-metis",
        "name": "ultra-metis"
      },
      {
        "path": "/Users/dan/Code/other-project",
        "name": "other-project"
      }
    ]
  }
}
```

#### `bridge.welcome` (server → bridge)

Server acknowledges the bridge connection. If the protocol version is incompatible, the server responds with `"accepted": false` and an `error` message, then closes the connection.

```json
{
  "type": "bridge.welcome",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:29:01Z",
  "payload": {
    "accepted": true,
    "protocol_version": 1,
    "server_version": "0.1.0",
    "machine_id": "dans-macbook",
    "pending_requests": []
  }
}
```

The `pending_requests` array contains any `session.start` requests that were made while the bridge was disconnected. Each entry has the same shape as a `session.start` payload. The bridge should process these after sending its `session.sync` message.

Rejection example:

```json
{
  "type": "bridge.welcome",
  "id": "msg-uuid",
  "timestamp": "2026-03-19T14:29:01Z",
  "payload": {
    "accepted": false,
    "protocol_version": 1,
    "server_version": "0.2.0",
    "error": "Protocol version 0 is no longer supported. Please upgrade shepherd-bridge."
  }
}
```

## Agent Bridge

### Daemon Lifecycle

The bridge runs as a background daemon on the dev machine. It:

1. Starts on login (or manually via `shepherd-bridge start`)
2. Connects to the central server via WebSocket (reconnects on disconnect with exponential backoff)
3. Discovers available projects by scanning configured directories
4. Monitors for locally-started Claude Code sessions (see Session Detection below)
5. Manages remotely-started sessions via PTY allocation

CLI commands:

```
shepherd-bridge start              # Start daemon (foreground or daemonize)
shepherd-bridge stop               # Stop daemon gracefully
shepherd-bridge status             # Show connection status and active sessions
shepherd-bridge config             # Show/edit configuration
shepherd-bridge wrap <command>     # Run command in bridge-managed PTY
shepherd-bridge notify <event>     # Send event to running daemon (used by hooks)
```

### Configuration

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
# Directories to scan for projects (recursively, 2 levels deep)
scan_dirs = [
  "/Users/dan/Code"
]
# Or explicit project paths
# explicit = ["/Users/dan/Code/ultra-metis"]

[adapters.claude-code]
binary = "claude"           # Path or name on $PATH
default_model = "opus"
hook_pipe_dir = "/tmp/shepherd/hooks"
```

### Session Detection (Local Sessions)

When a user starts Claude Code directly in a terminal (not via the web UI), the bridge needs to detect it. Two mechanisms:

**Primary: Claude Code hook integration.** Configure a Claude Code hook (via `~/.claude/hooks.json` or project-level) that fires on session start and writes a registration message to a named pipe or Unix socket monitored by the bridge:

```json
{
  "hooks": {
    "SessionStart": [{
      "command": "shepherd-bridge notify session-start --project $CWD --pid $PPID"
    }]
  }
}
```

Note: `$CWD` and `$PPID` are examples — the actual environment variables depend on what Claude Code's hook system exposes. The bridge should document the expected hook configuration based on the Claude Code version. `$PPID` targets the Claude Code process (parent of the hook shell), not `$$` which would be the hook shell itself.

The bridge's `notify` subcommand sends a message to the running daemon via a local Unix socket. The daemon then begins monitoring that process via `ps` and tracks it as a known session.

**Fallback: Process polling.** Periodically scan for `claude` processes, match them to known project directories, and register them as sessions with `status: limited` (visible in the dashboard but no interaction relay).

### Session Management

The session manager handles two flows:

**Remote-started sessions (full capability):**
1. Server sends `session.start`
2. Bridge allocates a PTY via `openpty()`
3. Bridge spawns `claude` process in the PTY with the initial prompt piped to stdin
4. Claude Code adapter attaches to the PTY for output parsing and input injection
5. Bridge sends `session.register` to server

**Locally-started sessions via `wrap` (full capability):**
The recommended workflow for local sessions with full interaction relay:

```bash
# Instead of:
claude

# Use:
shepherd-bridge wrap claude

# Or alias in .zshrc:
alias claude='shepherd-bridge wrap claude'
```

This transparently wraps Claude Code in a bridge-managed PTY while keeping the terminal experience identical. The user sees normal Claude Code output in their terminal. The bridge simultaneously parses the output and relays interactions to the server.

The `wrap` implementation must handle:
- **Signal forwarding**: Ctrl+C (SIGINT), Ctrl+Z (SIGTSTP), and SIGWINCH (terminal resize) must be forwarded to the child process
- **Terminal capabilities**: Pass through the `TERM` environment variable and terminfo settings
- **Exit code propagation**: `wrap` exits with the same code as the wrapped process
- **Line editing**: The PTY handles readline/line editing natively; no special handling needed

**Hook-detected sessions (monitoring only):**
Sessions started without `wrap` are detected via hooks. These appear in the dashboard with status information but cannot relay interactions (no PTY ownership). The UI shows a note suggesting to use `wrap` for full capability.

### Adapter Layer

#### AgentAdapter Trait

```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Human-readable adapter name (e.g., "claude-code")
    fn agent_type(&self) -> &str;

    /// What interaction types this adapter can detect and relay
    fn capabilities(&self) -> AdapterCapabilities;

    /// Start a new session in a managed PTY
    async fn start_session(&self, config: SessionConfig) -> Result<SessionHandle>;

    /// Stop a running session (send SIGTERM, wait, SIGKILL)
    async fn stop_session(&self, handle: &SessionHandle) -> Result<()>;

    /// Subscribe to interaction events detected in session output
    fn interaction_stream(
        &self,
        handle: &SessionHandle,
    ) -> Pin<Box<dyn Stream<Item = InteractionPrompt> + Send>>;

    /// Inject a human response into the session's stdin
    async fn send_response(
        &self,
        handle: &SessionHandle,
        response: InteractionResponse,
    ) -> Result<()>;

    /// Subscribe to raw output stream (sanitized of ANSI codes).
    /// Post-MVP: not required for initial implementation.
    fn output_stream(
        &self,
        handle: &SessionHandle,
    ) -> Option<Pin<Box<dyn Stream<Item = OutputChunk> + Send>>>;
}

pub struct AdapterCapabilities {
    pub interactions: Vec<PromptType>,
    pub can_cancel: bool,
    pub supports_output_streaming: bool,
}

pub struct SessionConfig {
    pub project_path: PathBuf,
    pub initial_prompt: Option<String>,
    pub model: Option<String>,
    pub extra_args: Vec<String>,
}

pub struct SessionHandle {
    pub session_id: Uuid,
    pub pid: u32,
    pub pty_master: OwnedFd,  // Unix-only; see Platform Constraints
    pub started_at: DateTime<Utc>,
}
```

#### Claude Code Adapter

The adapter that knows how to manage Claude Code specifically.

**PTY Management:**
- Allocates PTY pair via `nix::pty::openpty()`
- Spawns `claude` process with the child side of the PTY as stdin/stdout/stderr
- Reads from the parent side in an async loop, feeding bytes to the prompt detector and output stream
- Writes to the parent side when injecting responses

**Prompt Detection (Hybrid Strategy):**

1. **Hook-based detection** for tool permissions:
   - Configure Claude Code hooks that write structured JSON to a named pipe:
     ```json
     {
       "event": "tool_permission",
       "tool": "Bash",
       "description": "git push origin main",
       "timestamp": "2026-03-19T14:32:00Z"
     }
     ```
   - The adapter reads from this pipe and emits `approval` interaction prompts
   - This is the most reliable detection method — hooks fire deterministically

2. **Pattern-based detection** for everything else:
   - Maintain a sliding window of recent terminal output (last 50 lines)
   - Apply regex patterns to detect prompt states:

   | Pattern | Prompt Type | Confidence |
   |---------|-------------|------------|
   | `Allow .+ to (run\|execute\|write\|read)` | `approval` | High |
   | `Do you want to proceed\?` / `Should I ` | `confirm` | High |
   | `^[A-Z]\)` repeated 2+ times near end of output | `choice` | Medium |
   | Line ending in `?` followed by input cursor wait | `freeform` | Medium |
   | No output for >10s after a question-like line | `freeform` (fallback) | Low |

3. **Idle detection** as safety net:
   - If no output for 30 seconds and the last significant output looks like a question, emit a generic `freeform` prompt with the last 20 lines as context
   - The web UI shows these as "Session may need attention" with a text input and the recent output for context

**Response Injection:**

The adapter translates `InteractionResponse` back into terminal input:

| prompt_type | injection |
|-------------|-----------|
| `approval` | Write `y\n` or `n\n` to PTY |
| `confirm` | Write `y\n` or `n\n` to PTY. The optional `comment` field in the response is stored server-side in interaction history only — it is **not** injected into the PTY. |
| `choice` | Write the selected key + `\n` (e.g., `A\n`) to PTY |
| `freeform` | Write the text + `\n` to PTY |

**ANSI Sanitization:**

Terminal output goes through a sanitizer before being sent to the server:
- Strip ANSI escape sequences (colors, cursor movement, screen clearing)
- Normalize line endings to `\n`
- Collapse excessive blank lines
- Truncate extremely long lines (>1000 chars)

## Central Server

### Overview

An Axum-based Rust server with three interfaces:

1. **WebSocket endpoint** (`/bridge`) — for bridge connections
2. **REST API** (`/api/...`) — for the web UI
3. **Static file serving** — serves the web UI SPA

### Configuration

```toml
# ~/.config/shepherd/server.toml

[server]
host = "0.0.0.0"
port = 8420

[database]
# Tilde is expanded to $HOME at startup
path = "~/.config/shepherd/server.db"

[bridge]
heartbeat_interval_seconds = 30
stale_session_timeout_seconds = 120
```

### WebSocket Hub

Manages bridge connections:

- Accepts WebSocket upgrades at `/bridge`
- Expects `bridge.hello` as first message, responds with `bridge.welcome`
- Routes messages between bridges and the interaction queue
- Handles reconnection: includes any pending `session.start` requests in `bridge.welcome.pending_requests`; expects a `session.sync` from the bridge to reconcile active session state
- Ping/pong keepalive every 15 seconds

### Session Registry

In-memory state tracking all known sessions across all bridges:

```rust
pub struct SessionRegistry {
    sessions: HashMap<Uuid, SessionRecord>,
    by_machine: HashMap<String, Vec<Uuid>>,
}

pub struct SessionRecord {
    pub session_id: Uuid,
    pub machine_id: String,
    pub agent_type: String,
    pub project_path: String,
    pub project_name: String,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub capabilities: Capabilities,
}

pub enum SessionStatus {
    Starting,
    Working,
    WaitingForInput,
    Idle,
    Completed,
    Crashed,
}
```

### Interaction Queue

Stores pending interactions waiting for human response:

```rust
pub struct InteractionQueue {
    pending: Vec<PendingInteraction>,
    history: VecDeque<CompletedInteraction>,
}

pub struct PendingInteraction {
    pub interaction_id: Uuid,
    pub session_id: Uuid,
    pub prompt: InteractionPrompt,
    pub received_at: DateTime<Utc>,
    pub timeout_at: Option<DateTime<Utc>>,
}
```

Persisted to SQLite so interactions survive server restarts. Active sessions and heartbeats are in-memory only (rebuilt on bridge reconnect).

### REST API

All endpoints return JSON. The web UI is the only consumer.

```
GET  /api/sessions                          → List all active sessions
GET  /api/sessions/:id                      → Session detail
POST /api/sessions                          → Launch new session (proxied to bridge)
DELETE /api/sessions/:id                    → Cancel session (proxied to bridge)

GET  /api/interactions                      → List pending interactions
GET  /api/interactions/:id                  → Interaction detail
POST /api/interactions/:id/respond          → Submit response

GET  /api/machines                          → List connected bridges/machines
GET  /api/machines/:id/projects             → List available projects on a machine

GET  /api/output/:session_id                → SSE stream of session output (post-MVP)
GET  /api/output/:session_id/recent         → Last N lines of output (post-MVP)

WS   /api/ws                                → WebSocket for real-time UI updates
                                              (new interactions, session status changes)
```

The `/api/ws` endpoint is separate from `/bridge` — it pushes UI-relevant events to the web app so the phone shows new interactions instantly without polling.

### UI WebSocket Events

```json
{ "event": "interaction.new", "data": { ... } }
{ "event": "interaction.resolved", "data": { "interaction_id": "..." } }
{ "event": "session.updated", "data": { ... } }
{ "event": "session.ended", "data": { ... } }
```

### Persistence

- **SQLite** (`~/.config/shepherd/server.db`):
  - Interaction history (prompts + responses + timestamps)
  - Session history (start/end times, projects, outcomes)
  - Machine registry (known machines, last seen)
- **In-memory only**:
  - Active WebSocket connections
  - Current session status (rebuilt from bridge heartbeats)
  - Output stream buffers (post-MVP)

## Web UI

### Technology

- **Framework**: Preact with TypeScript (smaller bundle, ideal for mobile-first PWA; compat layer available if React ecosystem libraries are needed)
- **Styling**: Tailwind CSS, mobile-first responsive design
- **Build**: Vite, output embedded in the server binary for single-binary deployment
- **Real-time**: WebSocket connection to `/api/ws` for instant updates
- **PWA**: Service worker + manifest for add-to-home-screen on phone

### Views

#### Interaction Queue (Main Screen)

The primary view. A vertical list of cards, one per pending interaction, sorted by newest first.

Each card shows:
- Session badge (project name, colored dot for session status)
- Interaction title
- Truncated context (expandable)
- Interaction-specific controls:
  - **Approval**: Two large buttons — Allow / Deny
  - **Confirm**: Allow / Deny, expandable detail, optional comment field
  - **Choice**: Tappable option cards, one per choice
  - **Freeform**: Text input with send button

A badge count on the browser tab / PWA icon shows pending interaction count.

When the queue is empty: "All clear. N sessions working." with a link to the dashboard.

#### Session Dashboard

Grid/list of all active sessions:
- Project name
- Agent type badge
- Status indicator (working / waiting / idle)
- Time since last activity
- Tap to open session detail

Grouped by machine when multi-machine is supported.

#### Session Detail

Single session view:
- Header: project name, status, uptime
- Recent milestones / activity summary (MVP: status + heartbeat info only; post-MVP: live output stream)
- Current pending interaction (if any) with inline response controls
- Interaction history for this session
- Stop session button

#### Session Launcher

Form to start a new session:
- Select machine (MVP: only one)
- Select project (dropdown from bridge's available projects)
- Optional initial prompt (text area)
- Agent type (MVP: only Claude Code)
- Launch button

## Repo Structure

New repo, separate from ultra-metis:

```
shepherd/
├── Cargo.toml                          # Rust workspace
├── Makefile
│
├── protocol/                           # Shared protocol types
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── envelope.rs                 # Message envelope, ID generation
│       ├── session.rs                  # Session lifecycle messages
│       ├── interaction.rs              # Interaction prompt/response types
│       ├── output.rs                   # Output streaming messages
│       └── bridge.rs                   # Bridge handshake messages
│
├── server/                             # Central server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                     # Axum server setup, config
│       ├── config.rs                   # Server configuration
│       ├── ws_hub.rs                   # Bridge WebSocket management
│       ├── ws_ui.rs                    # UI WebSocket push
│       ├── sessions.rs                 # Session registry
│       ├── queue.rs                    # Interaction queue
│       ├── api.rs                      # REST API handlers
│       ├── db.rs                       # SQLite persistence
│       └── static_files.rs            # Embedded SPA serving
│
├── bridge/                             # Local daemon
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                     # CLI entry point (start/stop/status/wrap)
│       ├── daemon.rs                   # Daemon lifecycle, signal handling
│       ├── config.rs                   # Bridge configuration
│       ├── session_manager.rs          # Start/stop/monitor sessions
│       ├── pty.rs                      # PTY allocation and management
│       ├── ws_client.rs               # WebSocket client with reconnect
│       ├── local_socket.rs            # Unix socket for notify commands
│       ├── process_monitor.rs         # Process scanning fallback
│       └── adapters/
│           ├── mod.rs
│           ├── traits.rs              # AgentAdapter trait definition
│           ├── claude_code.rs         # Claude Code adapter
│           ├── prompt_detector.rs     # Pattern matching engine
│           └── ansi.rs                # ANSI escape sanitizer
│
├── web/                                # Mobile-first SPA
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── api/
│       │   ├── client.ts              # REST API client
│       │   └── ws.ts                  # WebSocket connection manager
│       ├── stores/
│       │   ├── sessions.ts            # Session state
│       │   └── interactions.ts        # Interaction queue state
│       ├── views/
│       │   ├── Queue.tsx              # Interaction queue (main)
│       │   ├── Dashboard.tsx          # Session overview
│       │   ├── SessionDetail.tsx      # Single session + output
│       │   └── Launch.tsx             # New session form
│       └── components/
│           ├── InteractionCard.tsx     # Base card component
│           ├── ApprovalCard.tsx        # Yes/No buttons
│           ├── ConfirmCard.tsx         # Approve/Reject + comment
│           ├── ChoiceCard.tsx          # Option selection
│           ├── FreeformCard.tsx        # Text input
│           ├── NotificationCard.tsx    # Info display
│           ├── SessionBadge.tsx        # Status indicator
│           └── OutputView.tsx          # Terminal output display
│
└── docs/
    ├── protocol.md                     # Protocol specification
    └── architecture.md                 # Architecture overview
```

## MVP Scope

### In Scope

1. **Protocol crate** — all message types, serde serialization, validation
2. **Bridge daemon** — start/stop/status/wrap, WebSocket client, session manager, PTY management
3. **Claude Code adapter** — prompt detection (hybrid), response injection, ANSI sanitization
4. **Central server** — WebSocket hub, REST API, session registry, interaction queue, SQLite persistence, embedded SPA
5. **Web UI** — interaction queue, session dashboard, session launcher
6. **`shepherd-bridge wrap`** — transparent Claude Code wrapper for local sessions

### Out of Scope (Future)

- Push notifications (use polling/auto-refresh for MVP)
- Multi-machine support (single bridge connection for MVP)
- Authentication (localhost-only, no auth needed)
- Non-Claude-Code adapters (protocol supports it, only Claude Code for MVP)
- Session output streaming in web UI (reduces complexity; session detail shows status only)
- Session persistence across server restarts (sessions are ephemeral; only interaction history persists)

## Build & Deployment

### Single-Machine MVP

```bash
# Build everything
make build

# Outputs:
#   target/release/shepherd-server
#   target/release/shepherd-bridge

# Install
make install
# Copies to ~/.local/bin/

# Start server (runs on port 8420)
shepherd-server start

# Start bridge (connects to localhost:8420)
shepherd-bridge start

# Open on phone: http://<machine-ip>:8420
```

The web UI is embedded in the server binary via `rust-embed` — no separate static file serving needed.

### Development

```bash
# Server with hot reload
cargo watch -x 'run -p shepherd-server'

# Bridge
cargo run -p shepherd-bridge -- start

# Web UI dev server (proxies API to shepherd-server)
cd web && npm run dev
```

## Platform Constraints

The bridge daemon is **macOS and Linux only**. PTY management (`openpty`, `nix` crate), process monitoring (`ps`, `waitpid`), and Unix domain sockets are Unix-specific. Windows support is not planned. The server and web UI are platform-agnostic.

## Security Considerations

### MVP (localhost only)

- Server binds to `0.0.0.0:8420` (accessible on LAN)
- No authentication — anyone on the network can interact
- Acceptable for single-user home network

### Post-MVP

- **API key auth** — simple shared secret for bridge-to-server and web UI-to-server
- **TLS** — self-signed cert or Let's Encrypt for HTTPS/WSS
- **Session tokens** — web UI gets a session token after auth, included in all requests
- **Rate limiting** — prevent abuse of session launch endpoint

## Error Handling

### Bridge Disconnection

- Server marks all sessions from that bridge as `status: unknown`
- Web UI shows "Bridge disconnected" warning
- When bridge reconnects, it sends a session state sync (all currently running sessions)
- Server reconciles and clears stale sessions

### Session Crash

- Bridge detects process exit via PTY EOF or waitpid
- Sends `session.deregister` with `reason: crashed` and exit code
- Server moves any pending interactions for that session to "expired"
- Web UI shows crash notification

### Interaction Timeout

- If a prompt has `timeout_seconds` set and expires, server sends `interaction.timeout` to bridge
- Bridge adapter handles timeout based on prompt type
- MVP: no timeouts. All interactions wait indefinitely.

### Undetected Prompts

- The idle detection safety net (30s no output after question-like content) catches most missed prompts
- Web UI shows "Session may need attention" with recent output and a freeform text input
- User can always fall back to viewing raw output and typing a response

## Future Extensions

### Push Notifications
- Web Push API for browser notifications
- Optional webhook to Slack/Discord/ntfy for mobile push

### Multi-Machine
- Each bridge identifies with a unique `machine_id`
- Server routes `session.start` to the correct bridge
- Dashboard groups sessions by machine

### Agent Marketplace
- New adapters registered as plugins to the bridge
- Protocol stays the same — adapters translate to/from their agent's native interface
- Configuration per adapter type

### Session Templates
- Pre-configured session recipes ("Run tests on ultra-metis", "Deploy to staging")
- One-tap launch from the interaction queue's empty state

### Conversation History
- Store full sanitized output per session
- Searchable across sessions
- Link interactions to the output context they were generated from
