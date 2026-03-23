---
id: execution-records-audit-spine-and
level: initiative
title: "Execution Records, Audit Spine, and Traceability Index"
short_code: "SMET-I-0031"
created_at: 2026-03-16T20:06:13.526475+00:00
updated_at: 2026-03-17T00:59:12.694159+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: ultra-metis-core-engine-repo
initiative_id: execution-records-audit-spine-and
---

# Execution Records, Audit Spine, and Traceability Index

## Context

Every meaningful work run in Super-Metis should emit a durable ExecutionRecord that links intent to outcome. This is the audit spine of the system — without it, there's no way to answer "what happened, why, and with what evidence?" for any piece of work. Additionally, all durable objects need to be cross-linkable and queryable through a typed relationship index.

The product spec identifies ExecutionRecord as a core execution/traceability artifact alongside TransitionRecord, DecisionRecord, and the CrossReference index. Together they form the traceability backbone.

## Governing Commitments

- **Work is complete when required evidence exists** (Vision #14). Execution records ARE the evidence.
- **All durable project memory lives in the repo** (Vision #1). Execution history is persisted, not ephemeral.
- **Planning is durable and traceable from product intent to execution** (Vision #7). The cross-reference index makes traceability structural.

## Goals & Non-Goals

**Goals:**
- Implement ExecutionRecord domain type capturing: initiating artifact (Story/Task), execution mode, context sources used, architecture/rules consulted, notes fetched, tools run, files touched, validations run, durable artifacts updated, decisions made, escalations/overrides, final disposition
- Implement TransitionRecord domain type for phase transition audit trails: who, when, from-phase, to-phase, checks passed/failed, reason
- Implement DecisionRecord domain type for significant decisions: what was decided, alternatives considered, rationale, who approved, evidence
- Implement CrossReference index with typed relationships: parent/child, governs, references, derived-from, supersedes, conflicts-with, validates, blocks, approved-by
- Make all records queryable by document, date range, actor, type, and relationship
- SQLite indexing for efficient traceability queries
- MCP and CLI tools for querying execution history, transitions, decisions, and relationships
- Storage as markdown+frontmatter with SQLite as index

**Non-Goals:**
- Full event sourcing for all state changes — audit log captures essential history without that complexity
- Real-time streaming of execution events — records are written at completion or at significant milestones
- Building the runner that produces execution records — this initiative defines the record types and storage, not the execution engine

## Detailed Design

### ExecutionRecord Schema
```
ExecutionRecord {
  id: String,
  initiating_artifact: ShortCode,  // Story/Task that triggered this
  execution_mode: String,           // single-agent, orchestrated, manual
  started_at: DateTime,
  completed_at: Option<DateTime>,
  context_sources: Vec<ShortCode>,  // docs consulted
  architecture_consulted: Option<ShortCode>,
  rules_consulted: Vec<ShortCode>,
  notes_fetched: Vec<String>,       // note IDs
  tools_run: Vec<ToolEntry>,        // tool name, arguments, result summary
  files_touched: Vec<String>,       // file paths modified
  validations_run: Vec<ValidationEntry>,
  artifacts_updated: Vec<ShortCode>,
  decisions_made: Vec<ShortCode>,   // links to DecisionRecords
  escalations: Vec<EscalationEntry>,
  overrides: Vec<OverrideEntry>,
  final_disposition: String,        // completed, failed, blocked, abandoned
  notes: String,                    // free-form execution notes
}
```

### TransitionRecord Schema
```
TransitionRecord {
  id: String,
  document: ShortCode,
  from_phase: String,
  to_phase: String,
  actor: String,
  timestamp: DateTime,
  checks_run: Vec<CheckResult>,   // pre-transition hooks that ran
  reason: Option<String>,
  forced: bool,                   // was this a force override?
}
```

### DecisionRecord Schema
```
DecisionRecord {
  id: String,
  title: String,
  context: String,
  decision: String,
  alternatives: Vec<Alternative>,
  rationale: String,
  approved_by: Option<String>,
  evidence: Vec<ShortCode>,
  related_artifacts: Vec<ShortCode>,
  timestamp: DateTime,
}
```

### CrossReference Index
- Every durable object has a `references` field in frontmatter listing typed relationships
- Relationship types: parent/child, governs, references, derived-from, supersedes, conflicts-with, validates, blocks, approved-by
- SQLite maintains a materialized index of all relationships for efficient graph queries
- Queries supported: ancestors, descendants, siblings, governed-by, references, conflicts

### Storage
- Records stored in `.super-metis/execution/` subdirectories (runs/, transitions/, decisions/, trace-index/)
- Markdown+frontmatter format consistent with all other artifacts
- SQLite index for relationship graph and temporal queries

## Alternatives Considered

1. **Full event sourcing**: Deferred — over-engineering for now. Audit records capture essential history.
2. **Store only in SQLite (no files)**: Rejected — records must be repo-native and human-reviewable.
3. **Embed traceability in each document instead of a cross-reference index**: Rejected — a separate index is more queryable and doesn't bloat documents.

## Implementation Plan

Phase 1: Define ExecutionRecord, TransitionRecord, DecisionRecord Rust domain types
Phase 2: Implement serialization (markdown+frontmatter round-trip)
Phase 3: Implement CrossReference relationship types and index
Phase 4: Update SQLite schema for all record types and relationship index
Phase 5: Implement traceability queries (ancestors, descendants, relationships)
Phase 6: Integrate TransitionRecord emission into phase transition engine (coordinate with SMET-I-0007)
Phase 7: Add MCP and CLI tools for querying records and relationships
Phase 8: Integration tests for full traceability chains

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- ExecutionRecords can be created with all specified fields
- TransitionRecords are emitted automatically on every phase transition
- DecisionRecords capture decisions with rationale and evidence
- CrossReference index supports all typed relationships
- Traceability queries walk the full document graph in any direction
- All records round-trip through markdown+frontmatter without data loss
- SQLite queries return results in < 100ms for projects with up to 10,000 records
- MCP and CLI tools expose all query operations

## Risks / Dependencies

- Depends on SMET-I-0018 for planning types that records reference
- Depends on SMET-I-0007 for integration with phase transition engine
- Must coordinate with SMET-I-0029 (cognitive operations produce execution records)
- Must coordinate with SMET-I-0030 (notes fetched are tracked in execution records)
- Record volume could grow large — need retention/archival strategy

## Suggested Tasks for Decomposition

1. Define ExecutionRecord Rust domain type
2. Define TransitionRecord Rust domain type
3. Define DecisionRecord Rust domain type
4. Implement serialization for all record types
5. Design CrossReference relationship types and index schema
6. Implement CrossReference SQLite index and query engine
7. Integrate TransitionRecord emission into phase transition engine
8. Implement traceability queries (ancestors, descendants, graph walks)
9. Add MCP tools for record and relationship queries
10. Add CLI tools for record inspection and traceability