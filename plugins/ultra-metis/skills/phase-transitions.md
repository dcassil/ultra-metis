---
name: ultra-metis:phase-transitions
description: "This skill should be used when the user asks 'when to transition phases', 'move to active', 'exit criteria', 'what phase comes next', 'how to complete a task', 'transition to completed', 'mark as blocked', 'phase flow', or needs guidance on advancing documents through their lifecycle phases."
---

# Phase Transitions

This skill guides moving Ultra-Metis documents through their lifecycle phases.

## Phase Sequences

Phases move forward only. You cannot go backward to a previous phase. Terminal phases cannot transition further.

### ProductDoc
```
draft → review → published
```
- **draft**: Initial product definition
- **review**: Stakeholder feedback and refinement
- **published**: Stable product anchor, ready to drive work (terminal)

### Epic
```
discovery → design → ready → decompose → active → completed
```
- **discovery** → design
- **design** → ready
- **ready** → decompose
- **decompose** → active
- **active** → completed
- **completed**: terminal

### Story
```
discovery → design → ready → active → completed
                               ↓        ↓
                            blocked ←───┘
```
- **discovery** → design
- **design** → ready
- **ready** → active
- **active** → completed OR blocked
- **blocked** → ready OR active (return from blocked)
- **completed**: terminal

Note: Stories do NOT have a decompose phase — they are the implementation slices. Tasks under stories are created directly.

### Task
```
backlog → todo → active → completed
            ↓       ↓
         blocked ←──┘
```
- **backlog** → todo
- **todo** → active OR blocked
- **active** → completed OR blocked
- **blocked** → todo OR active (return from blocked)
- **completed**: terminal

### DesignContext
```
draft → review → published → superseded
```
- **draft** → review
- **review** → published
- **published** → superseded
- **superseded**: terminal

### ADR
```
draft → discussion → decided → superseded
```
- **draft** → discussion
- **discussion** → decided
- **decided** → superseded
- **superseded**: terminal

**WARNING**: Auto-advancing from `decided` moves to `superseded`. Most ADRs should stay in `decided` indefinitely. Only manually transition to `superseded` when explicitly replacing with a new ADR.

## Default Phases

When documents are created, they start in these phases:
- **ProductDoc**: `draft`
- **Epic**: `discovery`
- **Story**: `discovery`
- **Task**: `todo` (or `backlog`)
- **DesignContext**: `draft`
- **ADR**: `draft`

## Critical Rule: No Phase Skipping

**Transitions are constrained to adjacent phases only.**

Invalid transitions (will error):
- `todo → completed` (must go todo → active → completed)
- `discovery → active` (must progress through all intermediate phases)
- `draft → published` (must go draft → review → published)

**To complete a task**, call `mcp__ultra-metis__transition_phase` twice:
1. `transition_phase(short_code)` → todo to active
2. `transition_phase(short_code)` → active to completed

**To publish a ProductDoc**, call `mcp__ultra-metis__transition_phase` twice:
1. `transition_phase(short_code)` → draft to review
2. `transition_phase(short_code)` → review to published

## Using transition_phase

**Auto-advance (recommended):**
```
mcp__ultra-metis__transition_phase(short_code="PROJ-E-0001")
```
Moves to next valid phase. Validates exit criteria.

**Explicit phase (for blocked state):**
```
mcp__ultra-metis__transition_phase(short_code="PROJ-S-0042", phase="blocked")
```
Use explicit phase only for moving to/from blocked state (stories and tasks only).

**Force (use sparingly):**
```
mcp__ultra-metis__transition_phase(short_code="PROJ-E-0001", force=true)
```
Skips exit criteria validation. Use only when accepting the risk.

## Exit Criteria

Exit criteria are conditions that must be true before transitioning.

### Common Exit Criteria Patterns

**discovery → design (Epic/Story):**
- Problem statement clear and validated
- Key constraints identified
- Stakeholders aligned on scope

**design → ready (Epic/Story):**
- Solution approach documented
- Technical risks identified
- Dependencies mapped

**ready → decompose (Epic only):**
- Design reviewed and approved
- Team capacity available
- No blocking dependencies

**decompose → active (Epic):**
- Stories created with acceptance criteria
- Story backlog sufficient to start
- Team understands the work

**ready → active (Story):**
- Design approved, ready to implement
- Tasks identified if needed

**active → completed (Story/Task):**
- Acceptance criteria met
- Work verified/tested
- No known defects

## Blocked Work

Handle blocked work explicitly:

1. Transition to blocked: `mcp__ultra-metis__transition_phase(short_code, phase="blocked")`
2. Update document to record what's blocking
3. Address the blocker
4. Return from blocked: `phase="active"` or `phase="ready"` (stories) / `phase="todo"` (tasks)

**Note**: Only stories and tasks support the blocked state. ProductDocs, epics, DesignContexts, and ADRs cannot be blocked.

## Working in Active Phase

**CRITICAL**: Active stories and epics serve as persistent working memory. While in `active` phase, regularly update the document with:

- **Progress**: What's been completed
- **Findings**: Unexpected discoveries, blockers
- **Decisions**: Why you chose approach A over B
- **Next steps**: What remains if work is interrupted

Use `mcp__ultra-metis__edit_document` to update progress.

## When to Transition

### Pull-Based Transitions
- Move epic to **active** when capacity exists
- Move story to **active** when ready to implement
- Move to **completed** when actually done

### Don't Rush Transitions
Common mistakes:
- Epic in "active" with no stories isn't really active
- Story marked "completed" that doesn't meet criteria isn't done
- Design marked "ready" without review isn't ready

**The phases protect you.** They force discipline that prevents rework.
