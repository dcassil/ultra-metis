---
id: audit-smet-s-0001-core-initiatives
level: task
title: "Audit SMET-S-0001 Core Initiatives Against ADR SMET-A-0001"
short_code: "SMET-T-0157"
created_at: 2026-03-23T17:46:04.215558+00:00
updated_at: 2026-03-23T17:55:17.235999+00:00
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

# Audit SMET-S-0001 Core Initiatives Against ADR SMET-A-0001

## Objective

Review each non-completed initiative under the former SMET-S-0001 strategy against all 7 ADR SMET-A-0001 decision points. Update, annotate, or archive each initiative as needed.

### Initiatives to Audit

| Short Code | Title | Phase | Expected Impact |
|------------|-------|-------|-----------------|
| SMET-I-0009 | Extend MCP Tools | decompose | Scope — tool names change with rename |
| SMET-I-0010 | Extend CLI | design | Scope — binary/command names change |
| SMET-I-0017 | Monorepo Orchestration | discovery | Likely unaffected |
| SMET-I-0023 | Work Leasing | discovery | Defer — ADR uses simple claiming for MVP |
| SMET-I-0024 | Git Worktree Isolation | discovery | Archive or defer — ADR delegates to superpowers |
| SMET-I-0050 | Build/Release/Distribution | decompose | Scope — binary names change |
| SMET-I-0068 | Architecture Document Type | completed | Note — now dependency of I-0078 |
| SMET-I-0069 | Architecture Lifecycle Hooks | discovery | Dependency update — feeds into I-0078 |
| SMET-I-0070 | Architecture MCP Tools | discovery | Dependency update — feeds into I-0078 |
| SMET-I-0071 | Planning Data Views | discovery | Scope check — reflect new execution model data |
| SMET-I-0073 | Session-Scoped Ralph Loop | discovery | Note — ralph loop coexists with SDD per ADR |

### Per-Initiative Process
1. Read the initiative document
2. Check all 7 ADR decision points
3. Determine recommendation: keep as-is, update scope, update dependencies, archive, or merge
4. Edit document with "Cadre ADR Alignment" annotation if changes needed
5. Archive if fully superseded

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All 11 initiatives reviewed against ADR SMET-A-0001
- [ ] Each initiative has recommendation recorded
- [ ] Initiatives with scope/dependency changes have been edited
- [ ] Superseded initiatives archived with explanation
- [ ] Results recorded in Status Updates section below

## Status Updates

### Audit Results (2026-03-23)

| Code | Title | Recommendation | ADR Points | Action Taken |
|------|-------|---------------|------------|--------------|
| I-0009 | Extend MCP Tools | Update scope (rename) | 1 | Added alignment annotation. Mechanical rename by I-0074 |
| I-0010 | Extend CLI | Update scope (rename) | 1 | Added alignment annotation. Binary/command names change |
| I-0017 | Monorepo Orchestration | Keep as-is | — | Already NON-MVP/BACKLOG. No annotation needed |
| I-0023 | Work Leasing | Defer (confirmed) | 5 | Added alignment annotation. MVP uses simple claiming (I-0077). I-0024 archived |
| I-0024 | Git Worktree Isolation | **Archived** | 4 | Superseded by superpowers:using-git-worktrees delegation in I-0077 |
| I-0050 | Build/Release/Distribution | Update scope (rename) | 1 | Added alignment annotation. Artifact names change |
| I-0068 | Architecture Document Type | Keep as-is (completed) | — | No changes needed. Now dependency of I-0078 |
| I-0069 | Architecture Lifecycle Hooks | Update dependencies | 6,7 | Added alignment annotation. Now feeds into I-0078 Phase 4 |
| I-0070 | Architecture MCP Tools | Update deps + rename | 1,6 | Added alignment annotation. Dependency of I-0078, tool names change |
| I-0071 | Planning Data Views | Update scope | 1,3 | Added alignment annotation. Must show SDD execution data |
| I-0073 | Session-Scoped Ralph Loop | Complete | — | Added alignment annotation. Transitioning to completed |

**Summary**: 1 archived (I-0024), 1 completed (I-0073), 6 annotated with scope/dependency updates, 2 kept as-is, 1 deferred (confirmed).