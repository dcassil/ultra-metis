---
id: audit-and-align-existing
level: initiative
title: "Audit and Align Existing Initiatives with Cadre ADR Decisions"
short_code: "SMET-I-0079"
created_at: 2026-03-23T17:28:07.599908+00:00
updated_at: 2026-03-23T17:57:10.167531+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: audit-and-align-existing
---

# Audit and Align Existing Initiatives with Cadre ADR Decisions Initiative

## Context

ADR SMET-A-0001 made sweeping decisions that affect existing initiatives: rename to Cadre, superpowers as peer dependency, SDD-style execution, git worktree delegation, simple task claiming, architecture hooks as Phase 4. Without a systematic audit, the backlog contains stale initiatives with outdated assumptions.

## Goals & Non-Goals

**Goals:**
- Review every non-completed initiative under SMET-S-0001 and SMET-S-0002 against ADR SMET-A-0001
- For each: keep as-is, update scope, update dependencies, archive, or merge
- Update documents with findings
- Archive superseded initiatives
- Produce audit results summary table
- Ensure dependency graphs are consistent

**Non-Goals:**
- Executing scope changes (individual initiative responsibility)
- Auditing completed initiatives
- Modifying the ADR itself

## Detailed Design

### ADR Decision Points as Audit Criteria

| # | Decision Point | Impact Pattern |
|---|---------------|----------------|
| 1 | Rename to Cadre | Text updates in descriptions |
| 2 | Superpowers as peer dependency | No duplicate methodology work |
| 3 | SDD-style execution | Account for fresh-subagent model |
| 4 | Git worktree delegation | SMET-I-0024 likely archived |
| 5 | Simple claiming for MVP | SMET-I-0023 deferred |
| 6 | Architecture hooks as I-0078 deps | I-0069, I-0070 cross-referenced |
| 7 | SubagentStart hook | Initiatives involving agents should note this |

### Initiatives to Audit

**SMET-S-0001**: I-0009, I-0010, I-0017, I-0023, I-0024, I-0068, I-0069, I-0070, I-0071, I-0073
**SMET-S-0002**: I-0039 through I-0046

### Per-Initiative Process
1. Read current document
2. Check all 7 decision points
3. Determine recommendation
4. Edit document with "Cadre ADR Alignment" section if needed
5. Archive if superseded
6. Record in summary table

## Alternatives Considered

1. **Let initiatives self-correct when picked up**: Rejected — confusion compounds
2. **Archive all pre-ADR, re-create from scratch**: Rejected — destroys existing work
3. **Blanket "see ADR" note without analysis**: Rejected — reference without analysis isn't an audit

## Implementation Plan

1. Audit SMET-S-0001 core initiatives (1-2 days)
2. Audit SMET-S-0002 remote operations initiatives (1 day)
3. Compile summary table, verify dependency graph consistency (0.5 days)

Tasks 1 and 2 are parallelizable. Task 3 depends on both.

## Dependencies

- **Blocked by**: SMET-A-0001 (decided)
- **No blockers downstream** — this is a governance initiative

## Audit Summary (2026-03-23)

### S-0001 Core Initiatives

| Code | Title | Phase | Recommendation | ADR Points | Action |
|------|-------|-------|---------------|------------|--------|
| I-0009 | Extend MCP Tools | decompose | Update scope (rename) | 1 | Annotated. Mechanical rename by I-0074 |
| I-0010 | Extend CLI | design | Update scope (rename) | 1 | Annotated. Binary/command names change |
| I-0017 | Monorepo Orchestration | discovery | Keep as-is | — | Already NON-MVP/BACKLOG |
| I-0023 | Work Leasing | discovery | Defer (confirmed) | 5 | Annotated. MVP uses simple claiming (I-0077) |
| I-0024 | Git Worktree Isolation | discovery | **Archived** | 4 | Superseded by superpowers:using-git-worktrees |
| I-0050 | Build/Release/Distribution | decompose | Update scope (rename) | 1 | Annotated. Artifact names change |
| I-0068 | Architecture Document Type | completed | Keep as-is | — | No changes. Dependency of I-0078 |
| I-0069 | Architecture Lifecycle Hooks | discovery | Update dependencies | 6,7 | Annotated. Feeds into I-0078 Phase 4 |
| I-0070 | Architecture MCP Tools | discovery | Update deps + rename | 1,6 | Annotated. Dependency of I-0078 |
| I-0071 | Planning Data Views | discovery | Update scope | 1,3 | Annotated. Must show SDD execution data |
| I-0073 | Session-Scoped Ralph Loop | completed | **Completed** | — | Work done. Transitioned to completed |

### S-0002 Remote Operations Initiatives

| Code | Title | Phase | Recommendation | ADR Points | Action |
|------|-------|-------|---------------|------------|--------|
| I-0039 | Machine Connectivity and Trust | discovery | Update scope | 1,3 | Annotated. Execution model affects runners |
| I-0040 | Remote Session Lifecycle | discovery | Update scope | 1,3 | Annotated. Sessions involve subagent dispatch |
| I-0041 | Live Monitoring and Intervention | discovery | Update scope | 1,3 | Annotated. Monitor SDD execution |
| I-0042 | Notifications and Mobile Control | discovery | Update scope (rename) | 1 | Annotated. Rename only |
| I-0043 | Session History, Audit, and Replay | discovery | Update scope | 1,3 | Annotated. Audit covers SDD records |
| I-0044 | Policy and Safe Execution | discovery | Update scope | 1,3 | Annotated. Policies cover subagent dispatch |
| I-0045 | Cadre Work and Notes Integration | discovery | Update scope | 1,3 | Annotated. Rename + execution model |
| I-0046 | Operational Reliability | discovery | Update scope | 1,3 | Annotated. Multi-session = parallel subagents |

### Dependency Graph Verification

**New dependency chain** (from ADR SMET-A-0001 roadmap):
```
I-0074 (rename) → I-0075 (subagent awareness) → I-0076 (orchestrated execution) → I-0077 (parallel execution)
I-0069 + I-0070 + I-0076 → I-0078 (quality integration)
```

**Archived initiative check**: I-0024 was the only archival. I-0023 referenced I-0024 — updated its Risks/Dependencies section to note the archival.

**Totals**: 19 initiatives audited. 1 archived, 1 completed, 15 annotated, 2 kept as-is.