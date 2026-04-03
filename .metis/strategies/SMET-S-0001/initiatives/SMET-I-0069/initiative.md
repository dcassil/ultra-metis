---
id: architecture-lifecycle-hooks-for
level: initiative
title: "Architecture Lifecycle Hooks for Planning, Work, and Verification"
short_code: "SMET-I-0069"
created_at: 2026-03-18T19:17:42.219243+00:00
updated_at: 2026-03-18T19:17:42.219243+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#feature-architecture"
  - "#category-workflow-traceability"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: architecture-lifecycle-hooks-for
---

# Architecture Lifecycle Hooks for Planning, Work, and Verification

## Context

Cadre has a transition hook system (pre/post hooks with priorities, filtering, blocking capability) and an Architecture document type (SMET-I-0068). This initiative wires them together: registering three hooks that inject architecture context into the planning/work/verification lifecycle.

These hooks make architecture a living participant in the work lifecycle rather than an isolated reference document.

Design spec: `docs/superpowers/specs/2026-03-18-architecture-in-work-lifecycle-design.md`
Depends on: SMET-I-0068 (Architecture document type)

## Goals & Non-Goals

**Goals:**
- Implement Hook 1: Post-transition hook on Story → design that creates an Architecture document with context and checklist
- Implement Hook 2: Post-transition hook on Task → active that snapshots architecture context from parent Story's Architecture doc
- Implement Hook 3: Pre-transition hook on Story → completed that runs conformance check and blocks on drift
- Auto-create investigation Stories when conformance drift is detected
- Register all hooks in the HookRegistry with correct priorities and filters
- Integration tests for each hook and the full lifecycle flow

**Non-Goals:**
- The Architecture document type itself (covered by SMET-I-0068)
- MCP tools for architecture operations (covered by SMET-I-0070)
- Modifying the conformance checker logic (already exists)
- Modifying the transition hook system (already exists)
- Plugin-level skills or agents for architecture guidance

## Detailed Design

### Prerequisite: Hook System Refactor (HookContext)

The current hook closures (`PreTransitionCheck` and `PostTransitionAction`) only receive `&TransitionEvent` — they have no access to the document store, filesystem, or conformance checker. All three hooks need to read/write documents.

**Required change**: Refactor hook closure signatures to accept a `HookContext` parameter:

```rust
// New signature:
type CheckFn = Box<dyn Fn(&TransitionEvent, &dyn HookContext) -> PreCheckResult>;
type ActionFn = Box<dyn Fn(&TransitionEvent, &dyn HookContext) -> PostActionResult>;
```

`HookContext` trait provides:
- `read_document(short_code) -> Result<Document>`
- `create_document(params) -> Result<ShortCode>`
- `edit_document(short_code, search, replace) -> Result<()>`
- `find_children(parent_short_code, document_type) -> Result<Vec<ShortCode>>`
- `run_conformance_check() -> Result<ConformanceResult>`
- `project_path() -> &Path`

This is a breaking change but safe — no external consumers currently register hooks. The `TransitionEnforcer` already has access to the document store; this refactor threads that access through to hook closures.

This must be the first task in this initiative.

### Hook 1: Architecture Document Creation (Story → Design)

**Registration:**
- Type: PostTransitionAction
- Priority: GATE (200)
- Filter: document_type = Story, to_phase = Design

**Implementation:**
1. Via HookContext, query for ReferenceArchitecture documents with `ArchitectureStatus::Active`. If multiple, use most recently updated. If none, return success (no-op) — architecture is opt-in
3. Read the ReferenceArchitecture's full context: layers, boundaries, dependency rules, naming conventions, anti-patterns
4. Via HookContext, run conformance checker to capture baseline score (extracted from `ParsedToolOutput.summary["conformance_score"]`)
5. Via HookContext, read the Story document to get its `story_type` field
6. Generate checklist from the appropriate template for that Story type
7. Create a new Architecture document as a child of the Story:
   - source_reference_architecture = ReferenceArchitecture short code
   - Populate relevant_layers, relevant_boundaries, applicable_dependency_rules, applicable_naming_conventions, applicable_anti_patterns from ReferenceArchitecture
   - checklist = generated checklist items
   - locked = true
   - baseline_score = current conformance score
   - drift_tolerance = 0.02 (default)
8. Return PostActionResult with success and message including Architecture doc short code

### Hook 2: Architecture Context Snapshot (Task → Active)

**Registration:**
- Type: PostTransitionAction
- Priority: USER (500)
- Filter: document_type = Task, to_phase = Active

**Implementation:**
1. Find the Task's parent Story
2. Query for Architecture documents with parent_id = Story short code
3. If none found, return success (no-op)
4. Read the Architecture document
5. Append an `## Architecture Reference` section to the Task document containing:
   - Relevant layers and boundaries (formatted as bullet lists)
   - Applicable dependency rules
   - Applicable naming conventions
   - Applicable anti-patterns
6. This is a point-in-time snapshot, not a live reference
7. Return PostActionResult with success

### Hook 3: Conformance Gate (Story → Completed)

**Registration:**
- Type: PreTransitionCheck
- Priority: GATE (200)
- Filter: document_type = Story, to_phase = Completed

**Implementation:**
1. Query for Architecture document linked to this Story
2. If none exists, return PreCheckResult { passed: true } — no architecture gate
3. Read baseline_score and drift_tolerance from the Architecture document
4. Run ArchitectureConformanceChecker against current codebase
5. Calculate delta: current_score - baseline_score
6. If delta < -drift_tolerance (regression beyond tolerance):
   a. Via HookContext, create an investigation Story:
      - If original Story has an Epic parent: create under the same Epic
      - If no Epic parent: create as standalone backlog item with `backlog_category: "tech-debt"`
      - Title: "Investigate architecture drift from [original Story title]"
      - Type: investigation
      - Pre-populate context: which rules violated, boundaries crossed, score delta, original Story reference
   b. The new investigation Story will get its own Architecture document when it enters design (via Hook 1)
   c. Return PreCheckResult { passed: false, blocking: true, message: "Architecture conformance regressed..." }
7. If conformance maintained or improved:
   a. Update the Architecture document's completion_score
   b. Return PreCheckResult { passed: true }

### Hook Registration

All three hooks are registered during project initialization or MCP server startup via the HookRegistry:

```rust
registry.register_post_action(architecture_creation_hook);
registry.register_post_action(architecture_context_snapshot_hook);
registry.register_pre_check(conformance_gate_hook);
```

### Error Handling

- If ReferenceArchitecture read fails: hook returns success (degraded, not blocking)
- If Architecture document creation fails: hook returns failure (logged, non-blocking for Hook 1/2)
- If conformance checker fails: Hook 3 returns pass with warning (don't block Story completion on tool failure)
- All errors are logged to the transition's PostActionResult or PreCheckResult messages

## Alternatives Considered

1. **Plugin-first approach**: Build all integration in Claude Code plugin skills/hooks instead of Rust core. Rejected — only works for Claude Code users, not CLI. Architecture enforcement should be universal.
2. **Hybrid approach**: Core handles conformance checking, plugin handles intelligent parts (checklist generation, investigation guidance). Rejected — user chose hook-first for simplicity and consistency.
3. **Manual architecture document creation**: Require users to create Architecture documents themselves. Rejected — automation at the right lifecycle points is the core value proposition.

## Implementation Plan

Phase 0: Refactor hook system — define `HookContext` trait, update `PreTransitionCheck` and `PostTransitionAction` closure signatures, update `TransitionEnforcer` and `HookRegistry` to thread context through
Phase 1: Hook 1 — Architecture document creation on Story design transition
Phase 2: Hook 2 — Architecture context snapshot on Task active transition
Phase 3: Hook 3 — Conformance gate on Story completion with investigation auto-creation
Phase 4: Hook registration in HookRegistry during startup
Phase 5: Integration tests for each hook individually
Phase 6: End-to-end integration test for full lifecycle (Story design → Task active → Story complete)

## Acceptance Criteria

- [ ] `HookContext` trait defined with read_document, create_document, edit_document, find_children, run_conformance_check, project_path
- [ ] Hook closure signatures updated to accept `&dyn HookContext`
- [ ] `TransitionEnforcer` and `HookRegistry` thread context to hooks
- [ ] Existing hook tests updated and passing with new signatures
- [ ] Story entering design phase auto-creates an Architecture document with correct context and checklist
- [ ] Task entering active phase gets architecture context snapshot appended
- [ ] Story completion is blocked when conformance regresses beyond tolerance
- [ ] Investigation Story is auto-created on conformance drift with pre-populated context
- [ ] Story completion proceeds when conformance is maintained or improved, with completion_score recorded
- [ ] All hooks are no-ops when no ReferenceArchitecture exists (graceful degradation)
- [ ] All hooks are registered with correct priorities and filters
- [ ] Integration tests pass for individual hooks and full lifecycle

## Risks / Dependencies

- **Hard dependency on SMET-I-0068**: Architecture document type must exist before hooks can create them
- **Conformance checker accuracy**: False positives in conformance checking will block legitimate Story completions. Mitigation: drift_tolerance buffer, force transition option
- **Story scope detection**: Initially hooks use the full ReferenceArchitecture rather than scoping to relevant layers. Future improvement could use Story metadata about affected code areas.
- **Performance**: Conformance check at Story completion adds latency. Should be fast for typical repos but may need profiling for large codebases.

## Cadre ADR Alignment (SMET-A-0001)

**Audit date**: 2026-03-23 | **Recommendation**: Update dependencies

ADR point 6: This initiative is now a dependency of SMET-I-0078 (Quality Integration, Phase 4). The hooks implemented here are registered and wired into the execution flow by I-0078.

ADR point 7: The SubagentStart hook (SMET-I-0075) ensures subagents have Cadre context, which means architecture context snapshots (Hook 2) are available to subagents dispatched by `/cadre-execute`.

No scope changes needed — the hook designs remain correct. Only naming changes from the rename (I-0074) apply.