---
name: cadre:decomposition
description: "This skill should be used when the user asks to 'break down this epic', 'decompose into stories', 'create stories from epic', 'how to size stories', 'when to decompose', 'vertical slices', 'story granularity', or needs guidance on breaking higher-level work into lower-level work items."
---

# Work Decomposition

This skill guides the process of breaking higher-level work into actionable lower-level items.

## The Decomposition Chain

```
ProductDoc: "Make X a better experience"
    ↓
Epic: "User Authentication" (capability increment)
    ↓
Stories: "Login flow" (feature), "Password reset" (feature), "Fix token expiry" (bugfix)
    ↓
Tasks: "Implement OAuth callback", "Write migration script"
```

Each level breaks work above it into concrete, actionable pieces at appropriate scope.

## When to Decompose

Decompose **ahead of capacity**, not upfront:

- When team's current backlog is nearing its end
- During tail end of current work to prepare next batch
- When backlog is getting low (signal to look up and pull work down)

**Avoid**: Decomposing everything upfront (waterfall). Have work ready when capacity frees up, not entire project planned before starting.

## The Decompose Phase

Epics have an explicit "decompose" phase:

```
discovery → design → ready → decompose → active → completed
```

### Why Decompose is Explicit

The decompose phase creates a **visible buffer**:
- Designed epics waiting to be broken into stories
- Tracks how long things sit here
- Makes bottlenecks visible

**Don't skip to decompose early.** Premature decomposition leads to stories that solve wrong problems, rework when design changes, wasted effort.

## Sizing by Scope, Not Time

### Tasks: Atomic Execution Units
- **Scope**: A single, discrete execution step
- **Impact**: Moves the needle on parent story
- **Independence**: Can be worked without coordination
- **Examples**: "Implement OAuth callback handler", "Write DB migration", "Add unit tests for auth module"

### Stories: Implementation Slices
- **Scope**: An implementable slice typed by purpose
- **Impact**: Delivers a meaningful piece of the epic
- **Independence**: Can be worked and verified independently
- **Types**: feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup
- **Examples**: "Login flow" (feature), "Fix token expiry bug" (bugfix), "Migrate auth to JWT" (migration)

**If a story has meaningful sub-stories**, it should probably be an epic.

### Epics: Capability Increments
- **Scope**: Creates fundamental increment in capability
- **Impact**: Meaningfully changes what system can do
- **Coherence**: Stories within work toward unified outcome
- **Examples**: "User authentication", "Search functionality", "Billing integration"

**If it doesn't change what system can do**, it might just be a story.

## Decomposition Patterns

### Vertical Slices (Preferred)
Break by user-visible functionality:
```
Epic: "User authentication"
├── Story: "Login flow" (feature)
├── Story: "Registration flow" (feature)
├── Story: "Password reset" (feature)
└── Story: "Session management" (feature)
```
Each story delivers something user can see/use.

### Horizontal Layers (Use Sparingly)
Break by technical component:
```
Epic: "User authentication"
├── Story: "Database schema" (setup)
├── Story: "API endpoints" (feature)
├── Story: "Frontend components" (feature)
└── Story: "Integration tests" (setup)
```
Creates dependencies between stories. Prefer vertical slices.

### Risk-First
Break by unknowns:
```
Epic: "ML recommendation engine"
├── Story: "Evaluate model options" (investigation)
├── Story: "Build training pipeline" (feature, after investigation)
└── Story: "Integration with product" (feature)
```
Address risky/uncertain work first to fail fast.

### Milestone-Based
Break by deliverable checkpoints:
```
Epic: "Platform migration"
├── Story: "Read path on new platform" (migration)
├── Story: "Write path on new platform" (migration)
├── Story: "Deprecate old platform" (migration)
└── Story: "Cleanup and optimization" (refactor)
```
Each milestone independently valuable and deployable.

## Creating Stories from Decomposition

When decomposing an epic, use Cadre tools:

```
mcp__cadre__create_document(
  type="story",
  title="Login flow",
  parent_id="PROJ-E-0001",
  story_type="feature"
)
```

**CRITICAL**: After creating each story, immediately populate it with real content using `mcp__cadre__edit_document`. A story with template placeholders is incomplete and useless. Each story needs:
- Clear description of what to build/fix/change
- Acceptance criteria (how to know it's done)
- Story type rationale
- Any technical notes or constraints

## Quality Checklist

Good decomposition — each child item:
- **Independently valuable**: Delivers something useful alone
- **Clearly scoped**: Know when it's done
- **Right-sized**: Matches scope expectations for level
- **Aligned to parent**: Clearly contributes to level above

Bad decomposition smells:
- **Too granular**: "write line 42" — steps, not stories
- **Too vague**: "make it better" — no completion criteria
- **Wrong level**: Doesn't match document type scope
- **Orphaned**: Doesn't trace back to parent
- **Overlapping**: Multiple items covering same ground

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| Decomposing too early | Stories solve wrong problem | Stay in discovery/design until approach clear |
| Decomposing too late | Epic active with no stories | Decompose before moving to active |
| Wrong granularity | Stories that are epics or vice versa | Apply scope heuristics |
| Missing story types | All stories untyped | Choose appropriate type (feature, bugfix, etc.) |

## Judgment Calls

- **Uncertain scope?** Create investigation story first, then decompose based on findings
- **Large epic?** Consider if it's really multiple capability increments
- **Tiny epic?** Consider if it's really just a story
- **Cross-cutting?** May need stories under multiple epics, or a dedicated "platform" epic
