---
id: investigate-session-start-hook-sh
level: task
title: "Investigate session-start-hook.sh spawning orphaned processes"
short_code: "SMET-T-0220"
created_at: 2026-03-27T19:46:01.516354+00:00
updated_at: 2026-03-27T19:46:01.516354+00:00
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

# Investigate session-start-hook.sh spawning orphaned processes

## Objective

Investigate why `plugins/cadre/hooks/session-start-hook.sh` leaves orphaned shell processes that persist indefinitely after Claude sessions end. Observed 5+ zombie processes from a single day's usage.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All Cadre plugin users
- **Reproduction Steps**: 
  1. Start a Claude Code session in a Cadre project
  2. End the session
  3. Run `ps aux | grep session-start-hook`
  4. Observe orphaned processes still running
- **Expected vs Actual**: Hook processes should exit when the session ends. Instead they linger indefinitely, accumulating over time.

## Acceptance Criteria

- [ ] Root-cause why hook processes aren't exiting
- [ ] Fix the hook so processes clean up on session end
- [ ] Verify no orphaned processes after starting/stopping sessions
- [ ] Compare against architecture repo for drift

## Implementation Notes

### Files to Investigate
- `plugins/cadre/hooks/session-start-hook.sh`
- Plugin hook configuration in `plugin.json`

## Status Updates

*To be added during implementation*