---
id: cadre-plugin-guidance-agent
level: initiative
title: "Cadre Plugin: Guidance Agent"
short_code: "SMET-I-0065"
created_at: 2026-03-18T17:50:09.754763+00:00
updated_at: 2026-03-18T18:52:04.553290+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: cadre-plugin-guidance-agent
---

# Cadre Plugin: Guidance Agent

## Context

Metis has a `metis:flight-levels` agent — a specialized subagent that provides methodology guidance for document type selection, work decomposition, phase transitions, and Flight Levels best practices. This agent is spawned by Claude Code when users need help with Metis workflow decisions. It has access to Metis MCP tools and can read/search/create documents autonomously to provide contextual guidance.

Cadre has no equivalent agent. When users ask methodology questions ("what document type should I create?", "how do I decompose this initiative?", "when should I transition phases?"), there's no specialized agent to handle these queries using cadre tools.

**What Metis has:**
- `metis:flight-levels` agent with access to: Read, Grep, Glob, and all `mcp__metis__*` tools
- Triggers on: document type selection, work decomposition, phase transitions, Flight Levels best practices
- Provides autonomous methodology guidance using live project state

## Goals & Non-Goals

**Goals:**
- Create a guidance agent in `.claude-plugin/agents/` equivalent to Metis's flight-levels agent
- Agent should have access to cadre MCP tools for reading project state
- Agent should provide methodology guidance: document selection, decomposition approach, phase transition readiness
- Agent should be triggered automatically when users ask methodology questions
- Agent description should be specific enough to trigger on relevant queries without false positives

**Non-Goals:**
- Building multiple specialized agents (start with one comprehensive guidance agent)
- Task execution (that's SMET-I-0067's execution commands)
- Code generation or implementation work (agent is for methodology, not coding)
- Modifying cadre-core Rust code

## Detailed Design

### Agent File
```
.claude-plugin/
  agents/
    flight-levels.md    # Methodology guidance agent
```

### Agent Frontmatter
```yaml
---
name: cadre:flight-levels
description: Use this agent when working with Cadre documents and needing methodology guidance. Helps with document type selection, work decomposition, phase transitions, and Flight Levels best practices.
tools:
  - Read
  - Grep
  - Glob
  - mcp__cadre__list_documents
  - mcp__cadre__read_document
  - mcp__cadre__search_documents
  - mcp__cadre__create_document
  - mcp__cadre__edit_document
  - mcp__cadre__transition_phase
  - mcp__cadre__reassign_parent
  - mcp__cadre__archive_document
---
```

### Agent System Prompt Content
The agent's body should include:
- Cadre document type overview (vision, initiative, task, ADR, backlog) with when to use each
- Phase models for each document type with transition rules
- Decomposition guidance (vertical slices, sizing, dependency ordering)
- Project setup patterns (greenfield, tech debt, feature development)
- Human-in-the-loop rules for initiatives (check in before phase transitions)
- Common workflow examples

### Trigger Patterns
Agent should trigger when users ask about:
- "what document type should I create" / "should this be a task or initiative"
- "how do I break down this initiative" / "decompose into tasks"
- "when to transition phases" / "move to active" / "exit criteria"
- "how to set up a new project" / "which preset should I use"
- "how does cadre work" / "explain the workflow"

## Alternatives Considered

**Alternative 1: Multiple specialized agents (decomposition agent, phase agent, selection agent)**
- Rejected for now: Start with one comprehensive agent. Can split later if the system prompt becomes too large or triggers overlap.

**Alternative 2: No agent — rely on skills only**
- Rejected: Agents provide autonomous exploration of project state. Skills are static guidance. The agent can read current documents and give context-aware advice, while skills provide general methodology.

**Alternative 3: Copy the Metis flight-levels agent verbatim**
- Rejected: Need to adapt for cadre tool names, document types, and hierarchy. But the structure should be similar.

## Implementation Plan

1. Study Metis flight-levels agent structure and system prompt content
2. Create `.claude-plugin/agents/` directory
3. Write flight-levels agent with cadre tool access
4. Include methodology guidance adapted from Metis (document types, phases, decomposition, patterns)
5. Test agent triggering on relevant queries
6. Verify agent can read/query cadre project state

## Dependencies
- Depends on SMET-I-0064 (Skills) for consistent methodology content
- Should be developed after skills so guidance content is aligned

## Progress

### 2026-03-18 — Agent Created
Created `plugins/cadre/agents/flight-levels.md` with:
- Full frontmatter: name, description with 3 trigger examples, model=inherit, color=cyan
- Tool access: Read, Grep, Glob + all `mcp__cadre__*` tools
- Complete planning hierarchy documentation (ProductDoc → Epic → Story → Task)
- Document types table with phases, parents, and short codes
- Story types reference (8 types)
- Document selection decision tree
- User terminology mapping (bug → bugfix story, feature → story or epic, etc.)
- Document creation workflow (create → read → edit, never leave placeholders)
- Epic decomposition guidance (vertical slices, risk-first, milestone patterns)
- Phase transition guidance with common exit criteria
- Anti-patterns and principles
- Active document working memory pattern

## Exit Criteria
- [x] Agent created with proper frontmatter and tool access
- [x] Agent triggers on methodology queries (3 examples in description)
- [x] Agent can read cadre project state (all MCP tools in tool list)
- [x] Guidance content is consistent with skills from SMET-I-0064
- [x] Uses cadre hierarchy (ProductDoc/Epic/Story/Task, not Vision/Initiative/Task)