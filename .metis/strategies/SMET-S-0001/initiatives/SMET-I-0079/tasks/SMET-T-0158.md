---
id: audit-smet-s-0002-remote
level: task
title: "Audit SMET-S-0002 Remote Operations Initiatives Against ADR SMET-A-0001"
short_code: "SMET-T-0158"
created_at: 2026-03-23T17:46:16.460323+00:00
updated_at: 2026-03-23T17:56:16.924324+00:00
parent: SMET-I-0079
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0079
---

# Audit SMET-S-0002 Remote Operations Initiatives Against ADR SMET-A-0001

## Objective

Review each initiative under the former SMET-S-0002 (Remote AI Operations) strategy against ADR SMET-A-0001 decision points. All are in early discovery, so impact is likely limited to scope annotations about the new execution model and rename.

### Initiatives to Audit

| Short Code | Title | Expected Impact |
|------------|-------|-----------------|
| SMET-I-0039 | Machine Connectivity and Trust | Scope — execution model affects remote runners |
| SMET-I-0040 | Remote Session Lifecycle | Scope — sessions now involve subagent dispatch |
| SMET-I-0041 | Live Monitoring and Intervention | Likely minimal |
| SMET-I-0042 | Notifications and Mobile Control | Likely minimal |
| SMET-I-0043 | Session History, Audit, and Replay | Scope — audit must cover SDD execution records |
| SMET-I-0044 | Policy and Safe Execution | Likely minimal |
| SMET-I-0045 | Ultra-Metis Work and Notes Integration | Rename — ultra-metis refs become cadre |
| SMET-I-0046 | Operational Reliability and Multi-Session | Scope — multi-session means parallel subagents |

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All 8 S-0002 initiatives reviewed against ADR SMET-A-0001
- [ ] Each initiative has recommendation recorded
- [ ] Rename annotations added where needed
- [ ] Results recorded in Status Updates section below

## Status Updates

### Audit Results (2026-03-23)

All 8 initiatives under SMET-S-0002 reviewed against ADR SMET-A-0001. Each initiative received a "Cadre ADR Alignment (SMET-A-0001)" section added before its Implementation Plan.

| Short Code | Title | Recommendation | ADR Points | Notes |
|---|---|---|---|---|
| SMET-I-0039 | Machine Connectivity and Trust | Update scope | #1, #3, #7 | Runner must support multi-subagent sessions, verify SubagentStart hook availability, rename to Cadre namespace |
| SMET-I-0040 | Remote Session Lifecycle | Update scope | #1, #3, #5, #7 | Most impacted — session state machine must accommodate SDD-style orchestrated execution with subagent dispatch; task claiming integration needed |
| SMET-I-0041 | Live Monitoring and Intervention | Update scope (minor) | #1, #3, #7 | Event model needs subagent identity for multi-subagent sessions; two-stage review events as distinct types |
| SMET-I-0042 | Notifications and Mobile Control | Keep as-is (rename only) | #1 | Notifications are agnostic to execution model — only rename needed |
| SMET-I-0043 | Session History, Audit, and Replay | Update scope | #1, #3, #6 | History must capture SDD execution structure (subagent boundaries, review verdicts); future architecture hook events noted |
| SMET-I-0044 | Policy and Safe Execution | Update scope (minor) | #1, #2, #3, #4 | Policy must govern superpowers skill invocations (esp. worktrees); enforce per-subagent in orchestrated execution |
| SMET-I-0045 | Ultra-Metis Work and Notes Integration | Update scope + rename | #1, #3, #6, #7 | Most rename-impacted — title and all MCP tool refs change; context loading must support per-subagent filtering; SubagentStart hook is the delivery mechanism |
| SMET-I-0046 | Operational Reliability and Multi-Session | Update scope | #1, #3, #4, #5 | Capacity model must consider subagent resource usage; worktree cleanup on failure; task claiming integration for concurrency |

### Summary

- **All 8 initiatives** need at minimum a rename annotation (#1)
- **6 of 8** need scope updates for the SDD execution model (#3)
- **Only SMET-I-0042** (Notifications) is unaffected beyond rename
- **SMET-I-0040** (Session Lifecycle) and **SMET-I-0045** (Work Integration) are the most impacted
- No initiatives need to be archived or merged — all remain valid under the new architecture
- All initiatives are in early discovery phase so impact is annotations only, no rework needed