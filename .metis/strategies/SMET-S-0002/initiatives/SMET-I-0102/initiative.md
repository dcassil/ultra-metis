---
id: agent-console-output-parsing-and
level: initiative
title: "Agent Console Output Parsing and Reply Input Fix"
short_code: "SMET-I-0102"
created_at: 2026-03-29T01:10:00.860661+00:00
updated_at: 2026-03-29T02:19:48.268987+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0002
initiative_id: agent-console-output-parsing-and
---

# Agent Console Output Parsing and Reply Input Fix Initiative

## Context

The dashboard's live agent console (session monitoring view) has two bugs that break the interactive session experience:

1. **Inconsistent output rendering**: Agent session events arrive in mixed formats — some as JSON objects and others as plain text strings. The dashboard is only rendering JSON-formatted events correctly; plain text responses are being lost/dropped, resulting in missing output in the console view.

2. **Reply input permanently disabled**: After the AI agent responds, the reply text field and action buttons remain disabled. The user cannot send follow-up messages or interact with the session, effectively making the console one-shot instead of interactive.

Both issues are in the control dashboard frontend, specifically in the session monitoring and intervention UI components.

## Goals & Non-Goals

**Goals:**
- Render all agent output formats correctly (JSON and plain text) in the console view
- Ensure the reply input field and buttons re-enable after receiving an agent response
- Maintain correct input state transitions throughout the session lifecycle

**Non-Goals:**
- Redesigning the console UI layout or styling
- Adding new output format types beyond what the API already sends
- Changes to the control API or machine runner event emission

## Discovery Findings

### Bug 1: Output Rendering — Claude JSON events rendered as raw strings

The supervisor runs `claude --print --output-format json -p "<instructions>"`. ALL stdout is JSON. The `LiveOutput` component only specially renders events where `content` parses as `{"type":"result",...}` (Claude result card). All other JSON lines (assistant messages, tool uses, system events) render as raw JSON strings via `categoryClasses()`.

Key files:
- `apps/control-dashboard/src/components/LiveOutput.tsx` — rendering logic
- `apps/machine-runner/src/supervisor.rs:100-103` — `--output-format json` flag
- `apps/machine-runner/src/output_capture.rs:154-188` — line classification

Additionally, the runner emits an injection confirmation event with `event_type: "injection"` (line 874 of runner.rs) which is NOT a valid `SessionOutputEventType` enum variant, causing the entire batch to fail deserialization at the API. While injection events are in separate batches from regular output, this is still a bug.

### Bug 2: Input Disabled — Session auto-terminates after process exits

The session runs `claude --print -p "<instructions>"` which processes one prompt and exits. When the process exits:
1. Supervisor sends `ProcessState::Stopped` → runner reports "stopped" to API
2. API marks session as terminal (`stopped`)
3. Dashboard polls session, `isTerminal = TERMINAL_STATES.includes('stopped')` → `true`
4. `GuidanceInput` receives `disabled={isTerminal}` → input field and buttons disabled

The process is dead, so even if the input were enabled, writing to stdin would fail. The fix requires changing session lifecycle to support continuation.

## Detailed Design

### Task 1: Parse and Render Claude CLI JSON Output

Add a `parseClaudeEvent` function in `LiveOutput.tsx` that understands Claude CLI JSON event types:
- `type: "assistant"` → extract `message.content[].text` and render as agent text
- `type: "tool_use"` / `type: "tool_result"` → render tool name + summary
- `type: "system"` → render system messages
- `type: "result"` → existing card rendering (already works)
- Fallback: render content as-is for unknown types

### Task 2: Fix Invalid "injection" Event Type

Change the runner's injection confirmation event from `event_type: "injection"` to `event_type: "output_line"` (or add `Injection` to the `SessionOutputEventType` enum if we want to distinguish them).

### Task 3: Re-enable Input for Completed Sessions via Continuation

When a session reaches a terminal state:
- Show a "Continue Session" button instead of fully disabling the input
- Clicking it creates a new session on the same machine/repo with context from the previous session
- The new session's instructions reference the original session for continuity

## Alternatives Considered

1. **Keep Claude process alive with interactive stdin** — Would require removing `-p` flag and using stdin for all prompts. Significant change to the supervisor and session lifecycle. Deferred for now.
2. **Just show "Session ended" message** — Simpler but doesn't solve the user's need to interact after the agent responds.
3. **Render raw JSON with syntax highlighting** — Better than plain text but still not user-friendly. Rejected in favor of semantic parsing.

## Implementation Plan

1. Fix LiveOutput rendering to parse Claude CLI JSON event types
2. Fix invalid "injection" event_type in machine runner
3. Add session continuation flow (input re-enable + new session creation)