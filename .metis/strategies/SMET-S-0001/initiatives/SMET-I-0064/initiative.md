---
id: ultra-metis-plugin-skills-and
level: initiative
title: "Ultra-Metis Plugin: Skills and Guidance Layer"
short_code: "SMET-I-0064"
created_at: 2026-03-18T17:50:08.486240+00:00
updated_at: 2026-03-18T18:02:03.201673+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: ultra-metis-plugin-skills-and
---

# Ultra-Metis Plugin: Skills and Guidance Layer

## Context

The original Metis plugin provides rich contextual guidance through skills: `decomposition` (how to break initiatives into tasks), `phase-transitions` (when/how to advance phases), `document-selection` (which document type to create), `project-patterns` (how to set up projects). These skills are one of Metis's strongest features — they make the system usable by providing in-context methodology guidance.

Ultra-Metis has no equivalent. The `.claude-plugin/` directory exists with only a `marketplace.json`. Without skills, agents using ultra-metis MCP tools must improvise methodology decisions instead of following structured guidance. This is the single highest-impact gap for usability.

**What Metis has that we need to port:**
- `metis:decomposition` — guidance on breaking initiatives into vertical-slice tasks, sizing, granularity
- `metis:phase-transitions` — when to transition, exit criteria per phase, what each phase means
- `metis:document-selection` — which document type fits (vision vs initiative vs task vs ADR vs backlog)
- `metis:project-patterns` — patterns for greenfield, tech debt, incident response, feature development
- `metis:help` — overview of the system and available commands

**Strategy**: Port these as ultra-metis plugin skills that reference ultra-metis MCP tools (`mcp__ultra-metis__*`) instead of metis tools (`mcp__metis__*`). Adapt content for ultra-metis's document types and phase models.

## Goals & Non-Goals

**Goals:**
- Create 5+ plugin skills in `.claude-plugin/skills/` covering core methodology guidance
- Port decomposition guidance adapted for ultra-metis document hierarchy
- Port phase-transition guidance for all ultra-metis document types and their phase models
- Port document-selection guidance for ultra-metis's document types (vision, initiative, task, ADR, backlog)
- Port project-patterns guidance for common work types
- Create a help/overview skill explaining the ultra-metis system
- All skills reference ultra-metis MCP tools, not metis tools
- Skills should be discoverable via Claude Code's skill system (proper frontmatter with descriptions)

**Non-Goals:**
- Building skills for execution workflows (ralph/decompose commands — covered by SMET-I-0067)
- Building new guidance not present in Metis (ultra-metis-specific methodology can come later)
- Modifying ultra-metis-core Rust code — this is purely plugin-level work
- Creating skills for the remote operations layer (SMET-S-0002)

## Detailed Design

### Plugin Directory Structure
```
.claude-plugin/
  marketplace.json          # Already exists
  skills/
    decomposition.md        # How to break work into tasks
    phase-transitions.md    # Phase lifecycle guidance per doc type
    document-selection.md   # Which document type to use
    project-patterns.md     # Patterns for common project types
    help.md                 # System overview and command reference
```

### Skill Frontmatter Pattern
Each skill needs YAML frontmatter for Claude Code's skill discovery:
```yaml
---
name: ultra-metis:decomposition
description: This skill should be used when breaking down initiatives into tasks...
---
```

### What to Port from Each Metis Skill

**decomposition.md**
- Vertical slice decomposition principles
- Task sizing guidance (1-14 days)
- When to decompose (initiative in decompose phase)
- How to identify task boundaries
- Adapted for ultra-metis hierarchy (Vision → Initiative → Task)

**phase-transitions.md**
- Valid phase sequences per document type
- Exit criteria for each phase
- When to check in with humans (initiatives)
- Auto-advance behavior
- Phase-specific guidance (what "discovery" means vs "design" vs "ready")

**document-selection.md**
- Decision tree: bug → backlog(bug), feature request → initiative or backlog(feature), tech debt → backlog(tech-debt), architecture decision → ADR, strategic direction → vision
- When to use each document type
- Examples of each type

**project-patterns.md**
- Greenfield project setup pattern
- Tech debt campaign pattern
- Feature development pattern
- Incident response pattern
- Each pattern: which documents to create, which preset to use

**help.md**
- Ultra-metis system overview
- Available MCP tools and what they do
- Available skills and commands
- Common workflows (create project, track work, execute tasks)

## Alternatives Considered

**Alternative 1: Build skills into ultra-metis-core as generated system prompts**
- Rejected: Skills are a plugin concern, not a core engine concern. Plugin skills are easier to iterate on (just edit .md files) and follow the established Claude Code plugin pattern.

**Alternative 2: Copy Metis skills verbatim and change tool names**
- Rejected partially: The guidance content should be adapted for ultra-metis's specific document types and phase models, not just find-and-replace on tool names. But the structure and methodology should be preserved.

**Alternative 3: Build entirely new guidance from scratch**
- Rejected: Metis's guidance is battle-tested and works well. Port first, then evolve.

## Implementation Plan

1. Study existing Metis plugin skills to understand structure, frontmatter, and content patterns
2. Create `.claude-plugin/skills/` directory
3. Port `help.md` — system overview (simplest, establishes patterns)
4. Port `document-selection.md` — decision tree for document types
5. Port `phase-transitions.md` — phase guidance per document type
6. Port `decomposition.md` — initiative-to-task breakdown guidance
7. Port `project-patterns.md` — common project setup patterns
8. Test each skill triggers correctly in Claude Code
9. Verify all MCP tool references point to ultra-metis tools

## Progress

### 2026-03-18 — Skills Created
All 5 skills implemented in `plugins/ultra-metis/skills/`:
- `help.md` — System overview, tool reference, common workflows, presets
- `document-selection.md` — Decision tree, type reference, user terminology mapping, ADR test
- `phase-transitions.md` — Phase sequences for all 5 doc types, exit criteria, blocked state, transition rules
- `decomposition.md` — Vertical slices, sizing, patterns (risk-first, milestone-based), quality checklist
- `project-patterns.md` — Greenfield, tech debt, incident response, feature development, anti-patterns

All skills:
- Have proper YAML frontmatter with name and description
- Reference `mcp__ultra-metis__*` tools (not `mcp__metis__*`)
- Are adapted for ultra-metis document hierarchy
- Are consistent with each other on phase models, document types, and methodology

## Exit Criteria
- [x] All 5 skills created with proper frontmatter and descriptions
- [ ] Skills trigger correctly when relevant queries are made in Claude Code
- [x] All tool references use `mcp__ultra-metis__*` prefix
- [x] Guidance content is adapted for ultra-metis document types and phases
- [x] Skills are consistent with each other (no contradictory guidance)