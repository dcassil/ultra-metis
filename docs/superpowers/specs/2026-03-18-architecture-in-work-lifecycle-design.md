# Architecture in Planning/Work/Verification Lifecycle

**Date**: 2026-03-18
**Status**: Design
**Strategy**: SMET-S-0001 (Cadre Core Engine)

## Problem

Cadre has built the infrastructure for architecture management — ArchitectureCatalogEntry, ReferenceArchitecture, conformance checker, transition hooks, quality gates — but none of it is wired into the actual planning/work/verification lifecycle. Architecture exists as isolated types with no influence on how Stories are designed, how Tasks are executed, or how completion is verified.

## Design Decisions

- **Planning**: Architecture surfaces as context during Story design (passive reference + structured checklist)
- **Work**: Architecture context is fetched and snapshotted when Tasks go active
- **Verification**: Conformance check runs at Story completion; drift blocks completion and triggers investigation
- **Approach**: Hook-First — all integration through the Rust core transition hook system
- **Architecture Document**: New document type that captures the architecture context snapshot and checklist for a Story

## Prerequisites: Hook System Refactor

The current hook closures (`PreTransitionCheck` and `PostTransitionAction`) only receive `&TransitionEvent` — they have no access to the document store, filesystem, or conformance checker. All three hooks in this design need to read/write documents and run the conformance checker.

**Required change**: Refactor hook closures to accept a context parameter:

```rust
// Current signature (insufficient):
type CheckFn = Box<dyn Fn(&TransitionEvent) -> PreCheckResult>;

// Required signature:
type CheckFn = Box<dyn Fn(&TransitionEvent, &dyn HookContext) -> PreCheckResult>;
```

Where `HookContext` provides:
- `read_document(short_code) -> Result<Document>` — read any document
- `create_document(params) -> Result<ShortCode>` — create new documents
- `edit_document(short_code, search, replace) -> Result<()>` — edit documents
- `find_children(parent_short_code, document_type) -> Result<Vec<ShortCode>>` — query child documents
- `run_conformance_check() -> Result<ConformanceResult>` — run conformance checker
- `project_path() -> &Path` — access to project path

This is a breaking change to the hook system and should be the first task in SMET-I-0069. The `TransitionEnforcer` already has access to the document store — the refactor threads that access through to hook closures.

## New Document Type: Architecture

**Document type**: `architecture`
**Short code prefix**: `AR` (e.g., `PROJ-AR-0001`)
**No phases**. Created once, locked by default, explicitly unlockable.

### Document Trait Pattern

The Architecture document follows the governance type pattern (like `ArchitectureCatalogEntry` and `ReferenceArchitecture`) — it does NOT implement the full `Document` trait. It has its own `phase()` method that always returns a fixed value (e.g., `Phase::Published`) for compatibility with systems that expect a phase, but it does not participate in the phase transition system. The `DocumentType` enum gets a new `Architecture` variant with `short_code_prefix() -> "AR"` and an empty `valid_transitions_from()` (no transitions possible).

### Child Document Discovery

Architecture documents are discovered via their `parent_id` field. The implementation adds a `find_children_by_type(parent_short_code, document_type)` query to the document store, which scans the parent document's directory for child documents of the specified type. This query is also exposed through the `HookContext` trait for use by hooks.

### Multiple ReferenceArchitecture Handling

When querying for the project's ReferenceArchitecture, hooks filter by `ArchitectureStatus::Active`. If multiple Active references exist, use the most recently updated one. If none are Active, no-op.

### Investigation Story Fallback

When Hook 3 creates an investigation Story, it places it under the same Epic as the original Story. If the original Story has no Epic parent, the investigation Story is created as a standalone backlog item with `backlog_category: "tech-debt"`.

### Fields

Core fields (via DocumentCore):
- `parent_id` — Story short code this architecture context belongs to
- Standard metadata (title, short_code, created_at, updated_at, tags, archived)

Architecture-specific fields:
- `source_reference_architecture: Option<String>` — ReferenceArchitecture short code it was derived from
- `relevant_layers: Vec<String>` — subset of layers relevant to this Story's scope
- `relevant_boundaries: Vec<String>` — module boundaries in play
- `applicable_dependency_rules: Vec<String>` — dependency rules that apply
- `applicable_naming_conventions: Vec<String>` — naming conventions that apply
- `applicable_anti_patterns: Vec<String>` — anti-patterns to watch for
- `checklist: Vec<ChecklistItem>` — architecture-aware questions with answer fields
- `locked: bool` — defaults to `true`
- `baseline_score: Option<f64>` — conformance score at creation time
- `completion_score: Option<f64>` — conformance score at Story completion
- `drift_tolerance: f64` — defaults to 0.02 (2%)

### ChecklistItem Struct

```rust
pub struct ChecklistItem {
    pub question: String,
    pub answer: Option<String>,
    pub story_type_relevance: Vec<StoryType>,
}
```

### Lock/Unlock Model

- Created locked by default
- Unlock requires explicit action via MCP tool or CLI
- Unlock records: actor, timestamp, reason
- Re-locks automatically on save
- Edit history tracked via standard document updated_at

## Hook 1: Architecture Document Creation (Story Design)

**Trigger**: Post-transition hook when Story transitions to `design` phase.

**Registration**:
- Priority: `GATE` (200)
- Filter: `document_type = Story`, `to_phase = Design`
- Type: Post-transition action (non-blocking)

**Behavior**:
1. Read the project's published ReferenceArchitecture
2. If none exists, no-op (architecture enforcement is opt-in)
3. Extract full architecture context: layers, boundaries, dependency rules, naming conventions, anti-patterns
4. Run conformance checker to capture baseline score
5. Generate checklist based on Story type (see Checklist Templates below)
6. Create Architecture document as child of the Story
7. Populate with context slice, checklist, and baseline score

## Hook 2: Architecture Context at Task Start

**Trigger**: Post-transition hook when Task transitions to `active` phase.

**Registration**:
- Priority: `USER` (500)
- Filter: `document_type = Task`, `to_phase = Active`
- Type: Post-transition action (non-blocking)

**Behavior**:
1. Walk up from Task → parent Story
2. Find the linked Architecture document on the Story
3. If none exists, no-op
4. Append `## Architecture Reference` section to the Task document containing:
   - Relevant layers and boundaries
   - Applicable dependency rules and naming conventions
   - Applicable anti-patterns
5. This is a read-only snapshot, not a live link

## Hook 3: Conformance Check at Story Completion

**Trigger**: Pre-transition hook when Story attempts to transition to `completed` phase.

**Registration**:
- Priority: `GATE` (200)
- Filter: `document_type = Story`, `to_phase = Completed`
- Type: Pre-transition check (blocking)

**Behavior**:
1. Find the linked Architecture document for this Story
2. If none exists, check passes automatically
3. Run ArchitectureConformanceChecker against current codebase using ReferenceArchitecture
4. Compare conformance score against baseline_score stored on Architecture document
5. If conformance regressed beyond drift_tolerance:
   - Return blocking failure with details (score delta, violations, boundaries crossed)
   - Create an `investigation` type Story under the same Epic:
     - Title: "Investigate architecture drift from [Story title]"
     - Gets its own Architecture document with drift details
     - Pre-populated with: which rules violated, which boundaries crossed, score delta
6. If conformance maintained or improved:
   - Record completion_score on Architecture document
   - Return pass

## MCP Tool Surface

Three new tools:

### get_architecture_context
Read the Architecture document for a given Story or Task. For Tasks, walks up to the parent Story to find it.

**Parameters**:
- `project_path: String` — path to .metis directory
- `short_code: String` — Story or Task short code

**Returns**: Architecture document content or "no architecture context" if none exists.

### unlock_architecture
Unlock an Architecture document for editing.

**Parameters**:
- `project_path: String` — path to .metis directory
- `short_code: String` — Architecture document short code
- `reason: String` — why the unlock is needed

**Returns**: Confirmation with unlock record.

### check_conformance
Run conformance check on-demand, not tied to a phase transition.

**Parameters**:
- `project_path: String` — path to .metis directory

**Returns**: Conformance score, violations, warnings, comparison to baseline if ReferenceArchitecture exists.

## Checklist Templates by Story Type

### feature
- Does this introduce new cross-layer dependencies?
- Which module boundaries does this touch?
- Does this follow naming conventions for the affected layers?
- Are there anti-patterns relevant to this feature area?

### bugfix
- Does the fix respect existing layer boundaries?
- Could the root cause indicate an architecture violation?

### refactor
- Does this change any module boundaries?
- Does this alter dependency direction between layers?
- Should the ReferenceArchitecture be updated to reflect this refactor?

### migration
- Does this require updating the ReferenceArchitecture?
- Which tolerated exceptions does this affect?
- Are there new boundaries or layers introduced?

### architecture-change
- Full checklist (all questions from all types)
- What is the expected conformance impact?
- Does this require a new ADR?
- Should the ReferenceArchitecture be updated after this change?

### investigation / remediation / setup
- Minimal: layer boundaries and anti-patterns questions only.

Custom checklist items can be added via the unlock/edit flow.

## Integration Flow Summary

```
Story → design phase
  └─ Post-hook creates Architecture document (locked, with context + checklist + baseline score)

Task → active phase
  └─ Post-hook snapshots architecture context from parent Story's Architecture doc into task

Story → completed phase
  └─ Pre-hook runs conformance check
      ├─ Pass: records completion_score, transition proceeds
      └─ Fail: blocks transition, creates investigation Story with drift details
```

## Implementation Scope

### Initiative 1: Architecture Document Type
- New domain type in cadre-core
- Frontmatter template and content template
- Serialization (from_file/to_file, from_content/to_content)
- ChecklistItem struct and checklist templates per Story type
- Lock/unlock semantics
- Unit tests

### Initiative 2: Architecture Lifecycle Hooks
- Hook 1: Story design → create Architecture document
- Hook 2: Task active → snapshot architecture context
- Hook 3: Story completed → conformance check gate
- Hook registration in the HookRegistry
- Integration with existing ArchitectureConformanceChecker
- Investigation Story auto-creation on drift
- Integration tests

### Initiative 3: Architecture MCP Tools
- get_architecture_context tool
- unlock_architecture tool
- check_conformance tool
- Wire into cadre-mcp tool registry
- Integration tests

## Risks

- **Hook system refactor**: The current hook closures lack document store access. Refactoring the hook signature is a prerequisite and a breaking change. Mitigated by the fact that no external consumers currently register hooks — all hooks are internal.
- **Conformance checker accuracy**: If the checker produces false positives, Story completion gets blocked incorrectly. Mitigation: drift_tolerance buffer and the ability to force transitions.
- **Story scope detection**: The hook needs to know which architecture layers/boundaries are relevant to a Story. Initially this will use the full ReferenceArchitecture; scoping to relevant subset may require Story metadata about affected code areas. Hook 1 also needs to read the Story's `story_type` field to generate the correct checklist, which requires document store access (addressed by the HookContext refactor).
- **Performance**: Conformance check at Story completion adds latency to the transition. Should be fast for typical repos but may need async option for large codebases.
- **Conformance score extraction**: The baseline_score and completion_score are extracted from `ParsedToolOutput.summary["conformance_score"]` returned by `ArchitectureConformanceChecker::check()`.
