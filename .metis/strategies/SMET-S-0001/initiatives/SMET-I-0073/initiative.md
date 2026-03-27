---
id: session-scoped-ralph-loop-state-to
level: initiative
title: "Session-Scoped Ralph Loop State to Prevent Cross-Session Interference"
short_code: "SMET-I-0073"
created_at: 2026-03-23T15:30:19.614433+00:00
updated_at: 2026-03-23T17:55:35.080463+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: session-scoped-ralph-loop-state-to
---

# Session-Scoped Ralph Loop State to Prevent Cross-Session Interference Initiative

## Context

The cadre plugin (SMET-I-0067) implements Ralph loop execution commands that delegate to the **ralph-loop plugin** for iteration infrastructure. The ralph-loop plugin's stop hook already has session isolation (checks `session_id` in frontmatter vs hook stdin). However, cadre's setup scripts were writing `CLAUDE_CODE_SESSION_ID` (wrong variable name) instead of `CLAUDE_SESSION_ID`, and the SessionStart hook wasn't exporting the session ID at all.

### What Was Fixed (2026-03-23)

**Immediate fixes applied to source and installed cache:**

1. **SessionStart hook** (`plugins/cadre/hooks/session-start-hook.sh`): Added `CLAUDE_SESSION_ID` export via `CLAUDE_ENV_FILE` so Bash tool commands can access the session identity
2. **setup-cadre-ralph.sh** line 204: Fixed `CLAUDE_CODE_SESSION_ID` → `CLAUDE_SESSION_ID`
3. **setup-cadre-decompose.sh** line 171: Fixed `CLAUDE_CODE_SESSION_ID` → `CLAUDE_SESSION_ID`

**Upstream Metis fix** (separate, more involved): PR https://github.com/colliery-io/metis/pull/7 — Metis uses its own stop hook and state files, so it needed session-scoped filenames (`.claude/cadre-ralph-active-{SESSION_ID}.yaml`) with backwards-compatible fallback.

### Architecture Difference

Cadre delegates to the ralph-loop plugin which writes to `.claude/ralph-loop.local.md` and has its own session isolation in the stop hook. The fix was simpler here — just ensure `CLAUDE_SESSION_ID` is available and correctly referenced so the ralph-loop plugin can do its job.

### Remaining Work

If cadre ever adds its own stop hook (independent of ralph-loop plugin), it must implement session-scoped state files following the pattern from the upstream Metis fix.

## Goals & Non-Goals

**Goals:**
- `CLAUDE_SESSION_ID` exported via `CLAUDE_ENV_FILE` in SessionStart hook — DONE
- Setup scripts reference correct variable name (`CLAUDE_SESSION_ID` not `CLAUDE_CODE_SESSION_ID`) — DONE
- Ralph-loop plugin's stop hook correctly isolates sessions using `session_id` frontmatter field — already works
- If cadre adds its own stop hook in future, use session-scoped state filenames

**Non-Goals:**
- Per-subagent isolation (subagents share parent session_id — correct behavior)
- Migrating existing state files (cleaned up naturally)

## Detailed Design

### Current Architecture (post-fix)

Cadre delegates iteration to the **ralph-loop plugin** (from claude-plugins-official). The data flow:

1. **SessionStart hook** reads stdin JSON, extracts `session_id`, exports `CLAUDE_SESSION_ID` via `CLAUDE_ENV_FILE`
2. **Setup scripts** (`setup-cadre-ralph.sh`, `setup-cadre-decompose.sh`) write `.claude/ralph-loop.local.md` with `session_id: ${CLAUDE_SESSION_ID:-}` in YAML frontmatter
3. **Ralph-loop stop hook** reads `session_id` from frontmatter and compares against `session_id` from hook stdin — exits if mismatch

### Upstream Metis (different architecture)

Metis uses its own stop hook and state files. The fix there was more involved:
- Session-scoped filenames: `.claude/cadre-ralph-active-{SESSION_ID}.yaml`
- Stop hook does filename lookup by session_id first, then legacy fallback with ownership check
- PR: https://github.com/colliery-io/metis/pull/7

### Future: Own Stop Hook

If cadre ever replaces ralph-loop with its own stop hook, follow the Metis pattern:
- Session-scoped state filenames
- Stop hook extracts `session_id` from stdin and only matches its own state file
- Backwards-compatible fallback for legacy unscoped files

## Alternatives Considered

1. **PID-based scoping**: Rejected — Stop hooks run in new shell processes so `$$` doesn't match the parent
2. **Transcript path as proxy**: Hook receives `transcript_path` but setup scripts don't, making correlation fragile
3. **Lock files / flock**: Overcomplicated; doesn't solve the "wrong session reads state" issue

## Implementation Plan — COMPLETED

1. ~~Fix variable name typo in setup scripts~~ — DONE (2026-03-23)
2. ~~Add session ID export to SessionStart hook~~ — DONE (2026-03-23)
3. ~~Sync fixes to installed plugin cache~~ — DONE (2026-03-23)
4. ~~Verify with debug logging that session_id flows correctly~~ — DONE (2026-03-23)
5. Future: if adding own stop hook, implement session-scoped state filenames

## Cadre ADR Alignment (SMET-A-0001)

**Audit date**: 2026-03-23 | **Recommendation**: Complete (transition to completed)

All work is done. The session-scoped state fix is verified and working. Per ADR decision #2, the ralph loop coexists with the new SDD-style execution — so this fix remains relevant for `/cadre-ralph` single-task iteration.

The rename (I-0074) will update the SessionStart hook and setup scripts with new paths/names, but the session_id mechanism is unchanged.