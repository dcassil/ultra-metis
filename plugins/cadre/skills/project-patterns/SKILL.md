---
name: cadre:project-patterns
description: "This skill should be used when the user asks to 'start a new project', 'greenfield project', 'tech debt campaign', 'incident response', 'feature development', 'initialize cadre', 'set up project', or needs guidance on project setup and applying patterns for different work types."
---

# Project Patterns

This skill guides project setup and provides patterns for different types of work.

## Cadre Planning Hierarchy

```
ProductDoc → Epic → Story → Task
```

- **ProductDoc**: Defines WHY the product exists (draft → review → published)
- **Epic**: Major capability increments (discovery → design → ready → decompose → active → completed)
- **Story**: Typed implementation slices (discovery → design → ready → active → completed)
- **Task**: Atomic execution units (backlog → todo → active → completed)

Supporting documents:
- **DesignContext**: Approved UI patterns and design specs
- **ADR**: Architecture decisions with alternatives and rationale

## Greenfield Projects

Starting a new project from scratch.

### When to Use
- New product or system
- Major rewrite (treat as new)
- Proof of concept → production
- New team forming

### The Pattern

**1. Initialize:**
```
mcp__cadre__initialize_project(project_path="/path/to/project/.metis", prefix="PROJ")
```
Choose short (2-8 char, uppercase), memorable, unique prefix.

**2. Define ProductDoc:**
Create ProductDoc answering: "Why does this product exist?"
- Product intent and scope
- Success criteria
- Target audience and key benefits

Transition: draft → review → published

**3. Create Initial Epics:**
Common greenfield epics:
- **Foundation/Setup**: Dev environment, CI/CD, architecture
- **Core Feature**: Main thing this product does
- **Integration**: Connecting to other systems
- **Release/Launch**: Getting to production

Don't create all epics upfront. Create enough to start, pull more as backlogs empty.

**4. Decompose and Execute:**
For each epic: discovery → design → decompose into stories → execute stories

### Greenfield Tips
- **Start small**: Maximum uncertainty, learn as you go
- **Foundation first**: But don't gold-plate it
- **Expect pivots**: Early epics may invalidate later plans
- **ADRs from day one**: Capture decisions while context fresh

## Tech Debt Campaigns

Systematic debt reduction.

### When to Use
- Accumulated technical debt affecting velocity
- Scheduled "debt sprints"
- Pre-migration cleanup
- Quality improvement work

### The Pattern

**1. Create Epic for the campaign:**
```
mcp__cadre__create_document(type="epic", title="API Cleanup Campaign", parent_id="PROJ-PD-0001")
```

**2. Create Stories by debt type:**
```
mcp__cadre__create_document(type="story", title="Refactor auth middleware", parent_id="PROJ-E-0001", story_type="refactor")
```

**3. Execute:** Work through stories systematically

### Tech Debt Tips
- Don't boil ocean: Focused campaigns > everything at once
- Tie to value: Why does fixing this matter?
- Measure impact: Before/after metrics validate effort
- Prevent recurrence: ADRs for decisions that caused debt

## Incident Response

Handling urgent, unplanned work.

### When to Use
- Production incidents
- Critical bugs
- Security vulnerabilities
- Customer escalations

### The Pattern

**1. Immediate:** Create bugfix story for tracking:
```
mcp__cadre__create_document(type="story", title="INCIDENT: Service X down", parent_id="PROJ-E-0001", story_type="bugfix")
```

**2. Triage:** Work the incident, update story with findings

**3. Resolution:** Complete immediate fix, transition to completed

**4. Follow-up:** For systemic fixes, create an epic:
```
mcp__cadre__create_document(type="epic", title="Prevent X recurrence", parent_id="PROJ-PD-0001")
```

**5. Postmortem:** Create ADR if architectural decisions made

### Incident Tips
- Track immediately: Even if "just a quick fix"
- Separate fix from prevention: Quick fix now, proper fix later
- Don't skip follow-up: Epics for systemic improvements
- Document decisions: ADRs capture why you chose approach

## Feature Development

Standard feature flow.

### When to Use
- New features planned in roadmap
- Enhancements to existing features
- Customer-requested functionality

### The Pattern

**1. Epic:** Create epic for the feature:
```
mcp__cadre__create_document(type="epic", title="User Dashboard", parent_id="PROJ-PD-0001", complexity="m")
```

**2. Discovery:** Understand requirements, constraints, users

**3. Design:** Define solution approach, create ADRs for decisions, link DesignContext references

**4. Decompose:** Break into typed stories (prefer vertical slices)

**5. Execute:** For each story: discovery → design → implement → test → complete

**6. Complete:** Verify acceptance criteria, transition epic to completed

### Feature Tips
- Discovery matters: Understand before designing
- Vertical slices: User-visible increments over technical layers
- Story types: Use the right type (feature, bugfix, refactor, etc.)
- Exit criteria: Define "done" clearly
- Iterate: Ship incrementally, get feedback

## Anti-Patterns to Avoid

| Anti-Pattern | Problem | Fix |
|--------------|---------|-----|
| **Shadow work** | Work outside system | Track everything in Cadre |
| **Too many active** | Context switching | Limit WIP; finish before starting |
| **Orphaned work** | No value alignment | Connect to parent or question value |
| **Skipping phases** | Problems found late | Respect exit criteria |
| **Premature decomposition** | Wrong stories | Stay in discovery/design first |
| **Untyped stories** | Lost context | Always set story_type |
| **Stale work** | Clutter | Archive unused items regularly |

## Core Principles

- **Work is pulled, never pushed**: Low backlog signals to look up
- **All work traces to ProductDoc**: If it doesn't align, question value
- **Phases exist for a reason**: Don't skip them
- **Filesystem is truth**: Documents are repo-native markdown+YAML
- **Scope over time**: Size by capability, not duration
- **Static tools first**: Prefer deterministic tools over unconstrained reasoning
