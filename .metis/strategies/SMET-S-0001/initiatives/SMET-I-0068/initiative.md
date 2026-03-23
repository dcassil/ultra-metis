---
id: architecture-document-type-and
level: initiative
title: "Architecture Document Type and Domain Model"
short_code: "SMET-I-0068"
created_at: 2026-03-18T19:17:40.954879+00:00
updated_at: 2026-03-20T17:00:32.752335+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: architecture-document-type-and
---

# Architecture Document Type and Domain Model

## Context

Ultra-Metis needs a new `Architecture` document type that captures architecture context and a structured checklist for Stories. When a Story enters its design phase, the system creates an Architecture document as a child of the Story, populated with the relevant slice of the project's ReferenceArchitecture and type-appropriate checklist questions.

The existing ArchitectureCatalogEntry and ReferenceArchitecture types provide the source data. This initiative creates the new document type that bridges architecture knowledge into the planning lifecycle.

Design spec: `docs/superpowers/specs/2026-03-18-architecture-in-work-lifecycle-design.md`

## Goals & Non-Goals

**Goals:**
- Implement the `Architecture` domain type in `ultra-metis-core` following existing document patterns
- Define `ChecklistItem` struct with question, answer, and story type relevance
- Implement checklist templates for all Story types (feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup)
- Implement lock/unlock semantics with audit trail (actor, timestamp, reason)
- Add baseline_score, completion_score, and drift_tolerance fields for conformance tracking
- Create frontmatter.yaml and content.md templates
- Implement from_file/to_file and from_content/to_content serialization
- Add short code prefix `AR`
- Comprehensive unit tests for creation, serialization, lock/unlock, checklist operations

**Non-Goals:**
- Phase lifecycle for Architecture documents (they have no phases)
- Wiring hooks that create Architecture documents (covered by SMET-I-0069)
- MCP tool exposure (covered by SMET-I-0070)
- Modifying ArchitectureCatalogEntry or ReferenceArchitecture types
- Conformance checking logic (already exists in `quality/conformance.rs`)

## Detailed Design

### Architecture Struct

New module at `src/domain/documents/architecture/mod.rs`:

```rust
pub struct Architecture {
    pub core: DocumentCore,
    pub source_reference_architecture: Option<String>,
    pub relevant_layers: Vec<String>,
    pub relevant_boundaries: Vec<String>,
    pub applicable_dependency_rules: Vec<String>,
    pub applicable_naming_conventions: Vec<String>,
    pub applicable_anti_patterns: Vec<String>,
    pub checklist: Vec<ChecklistItem>,
    pub locked: bool,
    pub baseline_score: Option<f64>,
    pub completion_score: Option<f64>,
    pub drift_tolerance: f64, // default 0.02
}

pub struct ChecklistItem {
    pub question: String,
    pub answer: Option<String>,
    pub story_type_relevance: Vec<StoryType>,
}
```

### Lock/Unlock Model

- `locked` defaults to `true` on creation
- Unlock records: actor (String), timestamp (DateTime), reason (String) stored as `unlock_history: Vec<UnlockRecord>`
- Re-locks automatically after edit via `to_file()`
- Attempting to edit a locked Architecture document returns an error

### Checklist Templates

Built-in templates keyed by StoryType. Each template returns a `Vec<ChecklistItem>` with pre-populated questions. The `story_type_relevance` field on each item allows cross-type questions (e.g., layer boundary questions appear in most types).

### Storage Pattern

Follows the governance type pattern (like RulesConfig, AnalysisBaseline):
- Own module directory with mod.rs, frontmatter.yaml, content.md
- Uses DocumentCore for shared fields
- Tera-based frontmatter rendering
- gray_matter for parsing
- Short code prefix: `AR`

### Document Trait Pattern

Architecture documents follow the governance type pattern (like ArchitectureCatalogEntry and ReferenceArchitecture). They do NOT implement the full Document trait. They have their own `phase()` method returning a fixed value (`Phase::Published`) for compatibility, but do not participate in the phase transition system. The `DocumentType` enum gets a new `Architecture` variant with `short_code_prefix() -> "AR"` and an empty `valid_transitions_from()`.

### Child Document Discovery

Architecture documents are discovered via their `parent_id` field. A new `find_children_by_type(parent_short_code, document_type)` query is added to the document store, scanning the parent document's directory for child documents of the specified type.

## Alternatives Considered

1. **Append architecture context to Story document**: Rejected — architecture context is substantial and benefits from being a first-class document with its own identity, linkability, and lock semantics.
2. **Use existing DesignContext type**: Rejected — DesignContext is for UI patterns and visual standards. Architecture context has different fields, semantics, and lifecycle (no phases, lock/unlock model).
3. **Architecture document with phases**: Rejected — the document represents a stable reference snapshot, not a workflow artifact. Lock/unlock is simpler and more appropriate.

## Implementation Plan

Phase 1: Architecture struct, ChecklistItem, UnlockRecord types with all fields
Phase 2: Checklist templates for all Story types
Phase 3: Frontmatter template and content template
Phase 4: from_file/to_file, from_content/to_content serialization
Phase 5: Lock/unlock logic with validation
Phase 6: Register `Architecture` variant in `DocumentType` enum with AR prefix and empty transitions
Phase 7: Implement `find_children_by_type` query on document store
Phase 8: Unit tests for all operations

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Architecture struct with all specified fields exists in ultra-metis-core
- [ ] ChecklistItem and UnlockRecord structs are defined
- [ ] Checklist templates exist for all 8 Story types
- [ ] Lock/unlock semantics work correctly with audit trail
- [ ] Serialization round-trips correctly (frontmatter + content)
- [ ] `Architecture` variant added to `DocumentType` enum with AR prefix and empty transitions
- [ ] `find_children_by_type` query works on document store
- [ ] Unit tests pass for creation, serialization, lock/unlock, checklist generation, and child discovery

## Risks / Dependencies

- Depends on existing DocumentCore, StoryType, and serialization patterns in ultra-metis-core
- Must coordinate with SMET-I-0069 on the interface for creating Architecture documents from hooks
- Checklist template quality directly affects planning value — templates should be reviewed after initial implementation