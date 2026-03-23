---
id: 001-cadre-execution-architecture-adopt
level: adr
title: "Cadre Execution Architecture: Adopt Superpowers Execution Patterns with Cadre State Backbone"
number: 1
short_code: "SMET-A-0001"
created_at: 2026-03-23T17:14:47.814053+00:00
updated_at: 2026-03-23T17:25:51.166921+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: Cadre Execution Architecture: Adopt Superpowers Execution Patterns with Cadre State Backbone

## Context

### The Problem

Ultra-Metis (being renamed to **Cadre** — "Constrained AI Developer Really Awesome Engine") needs an execution architecture that supports parallel multi-agent workflows with quality enforcement. The current architecture has fundamental limitations:

1. **Single long-running ralph loop** — All task execution happens in one session. Context pollution accumulates over iterations. No isolation between tasks. No review gates between tasks.

2. **No subagent awareness** — When Claude spawns subagents (via the Agent tool), those subagents have zero knowledge of Cadre's work management system. They don't know about documents, phases, MCP tools, or workflow rules. They operate as if Cadre doesn't exist.

3. **No parallel execution** — The `/ultra-metis-ralph-epic` command processes stories sequentially. Independent stories that could run concurrently are serialized, wasting time.

4. **No automated review** — Work quality depends entirely on the executing agent's discipline. There are no automated review checkpoints between tasks.

### The Comparison

A deep comparative analysis of ultra-metis vs the superpowers plugin (v5.0.5) revealed that the two plugins are naturally complementary, not competitive:

**Superpowers is a better executor.** Its subagent-driven-development (SDD) skill dispatches a fresh agent per task with precisely curated context, runs two-stage review (spec compliance + code quality) between tasks, supports model selection guidance (cheap models for mechanical work, capable for judgment), and uses git worktrees for isolation.

**Cadre is a better manager.** It has persistent repo-native state, enforced phase lifecycle (cannot skip phases), architecture and quality governance, deterministic story-type-to-skill mapping, cross-session working memory via documents, and audit/traceability records.

Neither plugin can do what the other does well. The question is how to combine them.

### Key Findings from Audit

| Facet | Cadre (Ultra-Metis) | Superpowers |
|-------|---------------------|-------------|
| Work tracking | Persistent documents (Vision/Strategy/Initiative/Task), repo-native, survives sessions | Ephemeral TodoWrite, dies with session |
| Task execution | Single ralph loop, context pollutes over time | Fresh subagent per task, isolated context |
| Review gates | None during execution | Two-stage: spec compliance then code quality |
| Lifecycle enforcement | Phase transitions enforced, cannot skip | Behavioral suggestions only |
| Architecture awareness | Full domain model, catalog, conformance | None |
| Quality baselines | Capture, compare, gate on degradation | None |
| Subagent context | SubagentStart hook missing — subagents are blind | SDD prompt templates curate exact context per agent |
| Git isolation | Not implemented (SMET-I-0024 deferred) | Full worktree skill with safety verification |
| Parallel dispatch | Sequential only | Independent-domain parallel dispatch |
| Cross-session memory | Documents as working memory, pre-compact restoration | Nothing — session death = total amnesia |
| Skill selection | Deterministic mapping: story_type → skills | Claude's judgment (frequently skips skills) |
| Model selection | Same model for everything | Cheap for mechanical, capable for judgment |

## Decision

**Adopt superpowers' execution patterns (fresh-subagent-per-task, two-stage review, worktree isolation) while using Cadre's state model as the persistent backbone. Keep superpowers as a peer dependency — do not absorb it.**

### Specifically:

#### 1. Keep Superpowers as a Dependency (Do Not Fork/Absorb)

Superpowers owns methodology skills: TDD, systematic debugging, brainstorming, verification-before-completion, writing-plans, code review patterns. These evolve independently and benefit all projects, not just Cadre-managed ones. Cadre should invoke them, not duplicate them.

#### 2. Replace Ralph Loop's Single-Session Model with SDD-Style Execution

The new execution command (`/cadre-execute` or similar) will:

- Read the epic/story, extract all tasks with full text (orchestrator curates context, not each agent)
- Dispatch a **fresh subagent per task** with precisely crafted context including:
  - Task content from the Cadre document
  - Mapped superpowers skills for the story type (deterministic, not optional)
  - Relevant code index sections
  - Architecture context if applicable
- Run **two-stage review** between tasks:
  - Spec compliance reviewer (separate subagent, distrusts implementer)
  - Code quality reviewer (separate subagent)
- Update Cadre documents with progress after each task completes
- Support **model selection**: cheap models for mechanical implementation tasks, capable models for judgment/integration tasks

#### 3. Add SubagentStart Hook for Context Injection

Every subagent spawned in a Cadre project receives:
- Awareness that this is a Cadre project with MCP tools available
- Current project state (active work items, blocked items)
- Instructions to update Cadre documents with progress
- The workflow rules (no TodoWrite, update documents as working memory)

This ensures even ad-hoc subagents (not just those dispatched by Cadre commands) know about and use the work management system.

#### 4. Delegate Git Worktree Isolation to Superpowers

When Cadre needs branch isolation for parallel work, invoke `superpowers:using-git-worktrees`. Do not reimplement. The full SMET-I-0024 (lease-triggered worktree creation) can build on this foundation later.

#### 5. Keep Ralph Loop for Single-Task Iteration

The existing ralph loop pattern (stop hook + state file + promise detection) remains for single-task iteration where fresh-subagent dispatch is overkill. The two models coexist:

| Scenario | Mechanism |
|----------|-----------|
| Single task, iterative | Ralph loop (existing) |
| Multi-task orchestrated | Cadre-execute with SDD-style subagent dispatch |
| Parallel independent tasks | Cadre-execute + worktree isolation |

#### 6. Cadre Documents Replace TodoWrite

Where superpowers' SDD uses TodoWrite for task tracking, Cadre uses its own documents. The orchestrator updates Cadre task documents with progress, findings, and status — not TodoWrite. This gives cross-session persistence and audit trail that TodoWrite cannot provide.

#### 7. Story-Type-to-Skill Mapping Remains Deterministic

Cadre's setup scripts deterministically map story types to superpowers skills. This is superior to superpowers' approach of letting Claude choose — Claude frequently skips skills it finds inconvenient. The mapping:

| Story Type | Required Skills |
|-----------|----------------|
| feature | brainstorming → writing-plans → TDD → verification |
| bugfix | systematic-debugging → verification |
| refactor | writing-plans → verification |
| migration | writing-plans → verification |
| architecture-change | brainstorming → writing-plans → verification |
| investigation | brainstorming |
| remediation | systematic-debugging → verification |
| setup | writing-plans → verification |

## Alternatives Analysis

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **A. Adopt superpowers execution + Cadre state** (chosen) | Best of both worlds. Fresh contexts, automated review, persistent state, enforced lifecycle. Maintains upstream superpowers compatibility. | Two plugin dependency. Must keep integration points maintained as superpowers evolves. | Low | Medium — SubagentStart hook + new execution command + worktree delegation |
| **B. Absorb superpowers into Cadre** | Full control. Single plugin. Can customize everything. | Massive scope increase. Diverges from upstream — miss improvements. Duplicates work. Methodology skills don't belong in an orchestration tool. | High | Very High — rewrite 12+ skills, maintain independently |
| **C. Keep current ralph loop, add review only** | Minimal change. Just add review checkpoints to existing loop. | Context pollution remains. No parallel execution. No model selection. Doesn't solve the core problem. | Low | Low — but solves only one gap |
| **D. Build custom execution engine from scratch** | Purpose-built for Cadre's needs. No dependency constraints. | Reinvents solved problems. Massive effort. No benefit over adopting proven patterns. | High | Very High |
| **E. Use superpowers SDD directly, skip Cadre orchestration** | Zero implementation work for execution. | Lose persistent state, lifecycle enforcement, deterministic skill mapping, architecture awareness, quality governance. Basically abandon Cadre's value proposition. | Medium | Zero (but loses Cadre's core value) |

## Rationale

Option A is chosen because:

1. **Natural boundary is clear**: Cadre owns state + lifecycle + governance. Superpowers owns methodology + execution patterns. Neither should do the other's job.

2. **Fresh-subagent-per-task is objectively better** than single long-running sessions for planned work. The evidence is clear from superpowers' design rationale and our own experience with context pollution in ralph loops.

3. **Two-stage review catches real issues** that single-agent self-review misses. The spec reviewer's explicit distrust of the implementer's report is a sound engineering practice.

4. **Maintaining superpowers as upstream** means we get improvements for free. The superpowers plugin is actively maintained by Anthropic. Absorbing it would mean maintaining 12+ skills ourselves.

5. **The integration surface is small**: SubagentStart hook, one new execution command, worktree delegation, document-instead-of-TodoWrite. These are well-bounded changes.

6. **Deterministic skill mapping is a differentiator** that Cadre should own. This is the orchestration layer's job — deciding which methodology applies to which work type.

## Consequences

### Positive

- **Parallel multi-agent execution** becomes possible with quality enforcement
- **Fresh agent contexts** per task eliminate context pollution in long executions
- **Automated two-stage review** (spec + quality) between tasks catches issues early
- **Persistent state** survives session crashes, context compaction, and agent handoffs
- **Deterministic skill application** ensures correct methodology is always used
- **Model selection** reduces cost for mechanical tasks
- **Subagent awareness** means every agent in a Cadre project knows the rules
- **Upstream compatibility** with superpowers means free improvements over time
- **Git worktree isolation** enables safe parallel work without conflicts

### Negative

- **Dual plugin dependency** — Cadre requires superpowers as a peer dependency. If superpowers changes its skill interfaces, Cadre's mappings must be updated.
- **SubagentStart hook adds latency** — every subagent spawn pays a small overhead for context injection. Should be fast (shell script reading a few files) but non-zero.
- **Complexity of orchestrated execution** — the new execution command is more complex than the current ralph loop. More moving parts means more failure modes.
- **superpowers version coupling** — Cadre's story-type-to-skill mapping references specific superpowers skill names. A rename or restructure in superpowers would break the mapping.

### Neutral

- Ralph loop continues to exist for single-task iteration — no migration required
- Existing commands (`/ultra-metis-ralph`, `/ultra-metis-decompose`) will be renamed to Cadre namespace but functionality is preserved
- The MCP server interface is unchanged — this decision affects plugin-level orchestration only

## Discussion Decisions (2026-03-23)

The following points were raised and resolved during discussion:

### 1. Rename Timing — Separate, Done First
The namespace rename (ultra-metis → cadre, `.metis` → `.cadre` for new projects) is its own initiative executed before any integration work. All subsequent work builds on the Cadre namespace from the start. The existing `.metis` folder in this project is NOT renamed — Cadre will be initialized independently as `.cadre`, tested in parallel, then docs migrated via script once proven.

### 2. Ralph Loop Coexistence — A/B Testable
Both execution models coexist and are A/B testable:
- `/cadre-ralph` — single task, ralph loop (existing mechanism)
- `/cadre-execute` — multi-task, SDD-style subagent dispatch (new)

Same task can be run both ways on separate branches. Cadre documents capture the evidence (progress, iterations, quality) for comparison. Ralph loop is not just transitional — it may remain the better choice for single-task iterative work where fresh-subagent overhead isn't justified.

### 3. Vendored Superpowers Fallback
Vendor the **entire** superpowers plugin (all 50+ files, all skills) into `vendor/superpowers/` in the Cadre repo. Not just the 7 skills Cadre currently maps to — the full plugin as a safety net.

Cadre's skill invocations try the installed plugin first (`superpowers:<skill>`). If not found (plugin removed, skill renamed, version mismatch), fall back to vendored copy. Resolution logic is a few lines of bash per invocation in setup scripts.

The vendor copy is pinned to a known-good version (currently 5.0.5) and updated deliberately, not automatically. This ensures Cadre is fully self-sufficient even if superpowers is uninstalled or undergoes breaking changes.

### 4. Simple Task Claiming for MVP, Full Leasing as Future Work
Phase 3 uses simple file-based task claiming (e.g., `.cadre/claims/PROJ-T-0001.lock` with session_id and timestamp). Sufficient for preventing duplicate work in multi-agent scenarios.

Full work leasing with timeouts, heartbeats, and stale lease recovery (SMET-I-0023) remains as future work. The simple claiming mechanism is designed to be replaceable by the lease system without changing the execution command interface.

### 5. Configurable Review Intensity — A/B Testable
The `/cadre-execute` command supports a `--review-mode` flag:

| Mode | Behavior | When to Use |
|------|----------|-------------|
| `full` (default) | Spec compliance + code quality review after every task | Production work, unfamiliar codebases |
| `light` | Code quality review only (skip spec compliance) | Well-specified tasks, trusted implementer model |
| `none` | No inter-task review | Rapid prototyping, XS tasks |

A/B testing: run the same initiative with each mode, compare defects caught, total token cost (from execution records), and time to completion. Cadre's audit trail captures everything needed for the comparison.

## Implementation Roadmap

### Phase 0: Rename (Prerequisite)
- Rename plugin namespace from ultra-metis to cadre
- Rename MCP server binary and CLI binary
- New projects use `.cadre/` directory (NOT renaming existing `.metis` folders)
- Update all plugin files: commands, hooks, scripts, skills, agents

### Phase 1: Subagent Awareness
- Implement SubagentStart hook that injects Cadre project context into all subagents
- Vendor 7 superpowers skills into `vendor/superpowers/` with fallback resolution
- Cadre documents replace TodoWrite in all orchestration flows

### Phase 2: Orchestrated Execution
- New `/cadre-execute` command using SDD-style fresh-subagent-per-task dispatch
- Story-type-to-skill deterministic mapping carried forward
- Two-stage review with `--review-mode` flag (full/light/none)
- Model selection guidance (cheap for mechanical, capable for judgment)
- Cadre document updates after each task completion
- A/B testing infrastructure: ralph loop vs cadre-execute on same tasks

### Phase 3: Parallel Execution
- Git worktree integration (delegate to `superpowers:using-git-worktrees`)
- Parallel dispatch for independent stories under an epic
- Simple file-based task claiming (`.cadre/claims/` with session_id)
- Full work leasing (SMET-I-0023) deferred as future enhancement

### Phase 4: Quality Integration
- Wire architecture lifecycle hooks (SMET-I-0069) into execution flow
- Conformance gates on story completion
- Quality baseline capture and comparison during review stages

## Related Documents

- SMET-I-0067: Ultra-Metis Plugin: Execution Commands (Ralph and Decompose) — completed, basis for current execution model
- SMET-I-0069: Architecture Lifecycle Hooks — planned, Phase 4 dependency
- SMET-I-0073: Session-Scoped Ralph Loop State — completed, fixes cross-session interference
- SMET-I-0023: Work Leasing and Ownership — deferred, future enhancement for Phase 3
- SMET-I-0024: Git Worktree Isolation for Leased Work — deferred, Phase 3 delegates to superpowers instead