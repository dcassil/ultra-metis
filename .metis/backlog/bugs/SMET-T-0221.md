---
id: investigate-subagent-start-hook-sh
level: task
title: "Investigate subagent-start-hook.sh spawning orphaned processes"
short_code: "SMET-T-0221"
created_at: 2026-03-27T19:46:31.695255+00:00
updated_at: 2026-03-27T19:46:31.695255+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Investigate subagent-start-hook.sh spawning orphaned processes

## Objective

Investigate why `plugins/cadre/hooks/subagent-start-hook.sh` leaves orphaned shell processes (paired, ~2 per invocation) that persist after subagents complete. Observed 6+ zombie processes from a single day's usage.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All Cadre plugin users
- **Reproduction Steps**: 
  1. Start a Claude Code session that spawns subagents
  2. Let subagents complete
  3. Run `ps aux | grep subagent-start-hook`
  4. Observe orphaned paired processes still running
- **Expected vs Actual**: Hook processes should exit when the subagent ends. Instead they linger indefinitely, accumulating over time.

## Acceptance Criteria

- [ ] Root-cause why hook processes aren't exiting
- [ ] Fix the hook so processes clean up on subagent completion
- [ ] Verify no orphaned processes after subagent lifecycle
- [ ] Compare against architecture repo for drift

## Implementation Notes

### Related
- SMET-T-0220 (same issue with session-start-hook.sh — likely same root cause)

### Files to Investigate
- `plugins/cadre/hooks/subagent-start-hook.sh`
- Plugin hook configuration in `plugin.json`

## Status Updates

*To be added during implementation*