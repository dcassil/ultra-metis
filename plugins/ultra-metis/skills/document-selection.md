---
name: ultra-metis:document-selection
description: "This skill should be used when the user asks 'what document type should I create', 'create a bug ticket', 'create a feature request', 'should this be a story or epic', 'when to use an ADR', 'track this bug', 'log this tech debt', or needs help choosing between product_doc, epic, story, task, design_context, or ADR document types."
---

# Document Type Selection

This skill helps choose the right Ultra-Metis document type for different kinds of work.

## Quick Decision Guide

**Is this work, or is it a decision?**
- Decision about architecture/approach → **ADR**
- Design spec or visual standard → **DesignContext**
- Work to be done → Continue below

**Does this define the product's purpose and direction?**
- Yes → **ProductDoc**

**Does this create a fundamental capability increment?**
- Yes → **Epic**

**Is this an implementable slice of work within an epic?**
- Yes → **Story** (choose appropriate story type)

**Is this an atomic execution unit?**
- Yes → **Task**

## Document Types Reference

| Type | Purpose | Parent Required | Short Code |
|------|---------|-----------------|------------|
| ProductDoc | Product definition | No | PD |
| Epic | Capability increments | ProductDoc (published) | E |
| Story | Typed implementation slices | Epic (decompose/active) | S |
| Task | Atomic execution units | Story | T |
| DesignContext | Design specs and patterns | No | DC |
| ADR | Architectural decisions | No | A |

## User Terminology Mapping

When users request work items using common terms, map to Ultra-Metis document types:

| User Says | Create |
|-----------|--------|
| "bug ticket", "bug", "defect" | Story (type=bugfix) under relevant Epic |
| "feature ticket", "feature request" | Story (type=feature) or Epic if large |
| "tech debt", "refactor" | Story (type=refactor) under relevant Epic |
| "project", "big feature", "capability" | Epic (under ProductDoc) |
| "investigation", "spike" | Story (type=investigation) |
| "small work item", "execution step" | Task (under Story) |

## When to Create Each Type

### ProductDoc
Create when:
- Starting a new project or product
- Redefining product direction
- Current product definition no longer represents objectives

**Not a ProductDoc**: "Build feature X" (that's an epic), "Fix bugs" (that's operational), "Q1 goals" (could be epics)

### Epic
Create when:
- Work delivers a meaningful capability increment
- Multiple stories needed to deliver it
- Discovery/design phases are valuable
- You want to track it as a distinct project

**Not an epic**: A single story (just make it a story), ongoing operations, an aspiration without commitment

### Story
Create when:
- Work is an implementable slice within an epic
- It has a clear type (feature, bugfix, refactor, migration, etc.)
- Discovery/design phases may be needed
- It's sized for meaningful delivery

**Story Types**: `feature`, `bugfix`, `refactor`, `migration`, `architecture-change`, `investigation`, `remediation`, `setup`

**Not a story**: Work too large to be a slice (that's an epic), a tiny execution step (that's a task)

### Task
Create when:
- Clear parent story exists
- Atomic execution unit
- One person can own it
- Done criteria are clear

**Not a task**: Work with no parent (needs a story/epic), work too large (break it down)

### DesignContext
Create when:
- Referencing approved UI patterns
- Linking design specs to implementation work
- Establishing visual standards for the product

### ADR
Create when:
- Making significant architectural decision
- Choosing between meaningful alternatives
- Decision affects multiple epics
- Future developers will wonder "why?"

**Not an ADR**: Trivial decisions, work to be done (story/task), meeting notes

**The ADR Test** — if 2+ of these are true, write an ADR:
1. Hard to reverse?
2. Affects multiple epics?
3. Evaluated meaningful alternatives?
4. Will outlive the current team?
5. Disagreement worth documenting?

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| Story that takes months | Wrong granularity | If it has sub-stories, make it an epic |
| Epic for every idea | Overhead | Keep in backlog, promote to epic when committed |
| ADR for implementation | Confusion | ADR records decision; stories/tasks implement it |
| Task without parent story | Orphaned work | Create the story first, then tasks under it |

## Edge Cases

**Story vs Epic**: Does it need multiple typed stories to deliver? If yes, epic. Is it a single implementable slice? Story.

**Story vs Task**: Does it need discovery/design phases? If yes, story. Is it a pure execution step? Task.

**Cross-cutting work**: Create epic under most relevant ProductDoc; stories can reference other epics in their documentation.
