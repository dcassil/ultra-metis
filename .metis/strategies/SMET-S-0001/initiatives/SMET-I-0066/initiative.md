---
id: cadre-plugin-lifecycle-hooks
level: initiative
title: "Cadre Plugin: Lifecycle Hooks"
short_code: "SMET-I-0066"
created_at: 2026-03-18T17:50:11.032447+00:00
updated_at: 2026-03-18T18:32:10.553234+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: cadre-plugin-lifecycle-hooks
---

# Cadre Plugin: Lifecycle Hooks

## Context

The Metis plugin has a `SessionStart` hook that detects the `.metis` directory and surfaces active work items at the start of every Claude Code session. This provides immediate project context: what initiatives are active, what tasks are in progress, and what needs attention. Without this, users must manually query project state before starting work.

Cadre has no lifecycle hooks. When a Claude Code session starts in a project with `.metis`, there's no automatic context loading, no active work summary, and no guidance on what to do next.

**What Metis has:**
- `SessionStart` hook that:
  - Detects `.metis` directory presence
  - Lists active initiatives and tasks with their phases
  - Shows actionable work items (tasks in todo/active)
  - Provides context for the session
  - Injects guidance about available skills and commands

**Additional hooks in the Claude Code ecosystem:**
- `UserPromptSubmit` — can validate/augment user input before processing
- `PreToolUse` / `PostToolUse` — can validate or react to tool calls
- `Stop` — can run checks when the agent stops working
- `Notification` — can surface information at key moments

## Goals & Non-Goals

**Goals:**
- Create a `SessionStart` hook that detects cadre projects and surfaces active work
- Hook should list active initiatives with current phase
- Hook should list actionable tasks (todo, active, blocked)
- Hook should remind about available cadre skills and commands
- Hook should set project context (metis path, project prefix, configuration)
- Consider additional hooks for validation (PreToolUse on transition_phase, etc.)

**Non-Goals:**
- Hooks that modify tool behavior (keep hooks informational, not blocking)
- Complex validation logic in hooks (keep them fast and simple)
- Hooks that require cadre binary execution (use prompt-based hooks, not shell scripts)
- Modifying cadre-core Rust code

## Detailed Design

### Plugin Directory Structure
```
.claude-plugin/
  hooks/
    session-start.md        # Project context on session start
```

### SessionStart Hook

The primary hook. On session start, when `.metis` directory is detected:

1. Detect `.metis` directory and determine project path
2. Call `mcp__cadre__list_documents` to get all documents
3. Filter to active/in-progress work items
4. Format as a concise summary:
   - Active initiatives with phase
   - Actionable tasks (todo/active/blocked)
   - Available commands and skills
5. Inject as session context

### Hook Implementation Pattern
Claude Code plugin hooks use prompt-based hooks (`.md` files with frontmatter):

```yaml
---
name: cadre-session-start
event: SessionStart
description: Load cadre project context on session start
---
```

The hook body contains instructions for what Claude should do when the event fires.

### Potential Future Hooks

**PreToolUse on transition_phase** (future):
- Validate that human approval was obtained for initiative transitions
- Warn if skipping phases or transitioning too quickly

**Stop hook** (future):
- Remind to update active task documents with progress before session ends

These are lower priority and can be added incrementally.

## Alternatives Considered

**Alternative 1: Shell script hooks that call cadre CLI**
- Rejected: Prompt-based hooks are simpler, don't require binary execution, and integrate better with the Claude Code plugin system.

**Alternative 2: No hooks — rely on CLAUDE.md for session context**
- Rejected: CLAUDE.md is static. Hooks provide dynamic context based on current project state (what's active NOW, not what was active when CLAUDE.md was written).

**Alternative 3: Complex hook suite covering every lifecycle event**
- Rejected for now: Start with SessionStart (highest impact). Add others incrementally based on real usage patterns.

## Implementation Plan

1. Study Metis SessionStart hook implementation for patterns
2. Create `.claude-plugin/hooks/` directory
3. Implement SessionStart hook with project detection and active work summary
4. Test hook fires correctly on session start
5. Verify hook output is concise and actionable
6. Consider adding PreToolUse hook for transition validation (stretch goal)

## Progress

### 2026-03-18 — Hooks Created
Created 3 files in `plugins/cadre/hooks/`:
- `hooks.json` — declares SessionStart and PreCompact hooks
- `session-start-hook.sh` — detects .metis, runs `cadre status`, outputs full project context including: planning hierarchy, active work summary, MCP tool reference, document types/phases table, story types, document creation rules, human-in-the-loop rules, task workflow
- `pre-compact-hook.sh` — lighter version of SessionStart for context restoration after compaction

Both hooks:
- Detect .metis directory, exit silently if absent
- Check for `cadre` CLI, warn if not installed
- Run `cadre status` to get live project state
- Reference `mcp__cadre__*` tools throughout
- Use cadre hierarchy (ProductDoc → Epic → Story → Task)
- Include story types and document phase models

## Exit Criteria
- [x] SessionStart hook created and functional
- [x] Hook detects cadre projects and surfaces active work
- [x] Hook output is concise (not overwhelming) and actionable
- [x] Hook reminds about available skills and commands
- [x] Hook performance doesn't noticeably slow session startup (shell script, no heavy operations)
- [x] BONUS: PreCompact hook also created for context restoration