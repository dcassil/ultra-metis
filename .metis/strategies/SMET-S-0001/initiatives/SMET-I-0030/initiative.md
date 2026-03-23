---
id: durable-insight-note-system
level: initiative
title: "Durable Insight Note System"
short_code: "SMET-I-0030"
created_at: 2026-03-16T20:06:12.211344+00:00
updated_at: 2026-03-17T00:48:37.546706+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: ultra-metis-core-engine-repo
initiative_id: durable-insight-note-system
---

# Durable Insight Note System

## Context

AI agents and developers accumulate local knowledge during work — hotspot warnings, misleading naming patterns, recurring bug signatures, validation hints, subsystem gotchas. Today this knowledge lives in chat context and is lost between sessions. Super-Metis needs a lightweight, self-pruning repo memory layer that captures this reusable local insight without becoming a transcript archive or a second documentation platform.

This is a first-class design concept in the product spec. Notes are the "B" category in the internal cognition vs durable persistence model: durable lightweight insight that sits between ephemeral reasoning and formal governed records.

## Governing Commitments

- **All durable project memory lives in the repo** (Vision #1). Notes are persisted as repo-native artifacts, scoped to local contexts.
- **Internal reasoning stays internal unless promoted** (Vision #9). Notes are the landing zone for promoted lightweight insight — confirmed, reusable, but not formal enough for governed records.
- **Only promote what matters** (Principle #7). The self-pruning mechanism ensures notes that don't prove useful are automatically candidates for removal.

## Goals & Non-Goals

**Goals:**
- Implement the DurableInsightNote domain type with full schema: id, title, note, category, tags, scope (repo/package/subsystem/paths/symbols), timestamps, fetch tracking, feedback counts, status (active/prune_candidate/needs_human_review/archived), review reason
- Implement note fetch by scope: at task start, fetch notes by repo/package/subsystem/path/symbol/symptom
- Implement fetch tracking: update last_fetched_at and increment fetch_count on each fetch
- Implement feedback scoring: at task wrap-up, record helpful/meh/harmful for each fetched note
- Implement automatic prune candidate detection: mark as prune_candidate when not fetched for threshold period, harmful ratio crosses threshold, meh accumulates without positive signal, or enough uses without demonstrated value
- Implement needs_human_review flagging: when note conflicts with architecture, conflicts with another note, may reflect stale architecture, may reflect stale note content, or may be risky to auto-prune
- Implement archival: move to archived when prune candidate long enough, explicitly replaced, or superseded
- Implement note creation and update workflows integrated into task wrap-up
- Add MCP and CLI tools for note CRUD, fetch, score, and inspection
- SQLite indexing for efficient scope-based note queries

**Non-Goals:**
- Storing temporary debugging thoughts, trace steps, or disposable sequencing decisions — these are ephemeral
- Building a full documentation platform — notes are compressed, local insight only
- Auto-generating notes from code analysis — notes are created from human/agent experience during work

## Detailed Design

### Note Schema (Rust Domain Type)
```
DurableInsightNote {
  id: String,
  title: String,
  note: String,
  category: String,
  tags: Vec<String>,
  scope: InsightScope { repo, package, subsystem, paths, symbols },
  created_at: DateTime,
  last_fetched_at: Option<DateTime>,
  fetch_count: u32,
  thumbs_up_count: u32,
  meh_count: u32,
  thumbs_down_count: u32,
  last_feedback_at: Option<DateTime>,
  status: NoteStatus (active | prune_candidate | needs_human_review | archived),
  review_reason: Option<ReviewReason>,
}
```

### What Notes Are For
- Hotspot warnings
- Recurring failure signatures
- Misleading names
- Validation hints
- Local exceptions
- Boundary warnings
- Subsystem quirks

### What Notes Are NOT For
- Temporary debugging thoughts
- Moment-by-moment search decisions
- Trace steps
- One-off relevance judgments
- Disposable sequencing decisions

### Behavioral Rules

**On fetch**: update last_fetched_at, increment fetch_count

**On task wrap-up**: for each fetched note, record helpful/meh/harmful feedback

**Prune candidate**: mark when not fetched for threshold period, harmful ratio crosses threshold, meh accumulates without positive signal, enough uses without demonstrated value

**Needs human review**: mark when conflicts with architecture, conflicts with another note, may reflect stale architecture/note, may be risky to auto-prune

**Archive**: when prune candidate long enough, explicitly replaced, or superseded

### Workflow Integration
- At task start: fetch notes by relevant scopes
- At task wrap-up: score fetched notes, propose new notes, propose note updates, flag risky conflicts for review

### Storage
- Stored as markdown+frontmatter in `.super-metis/memory/notes/`
- Indexed in SQLite for scope-based queries
- Queryable by scope, category, tag, status

## Alternatives Considered

1. **Use CLAUDE.md or memory files**: Rejected — no scope, no feedback, no self-pruning, no structured schema.
2. **Full knowledge graph**: Over-engineered — notes are lightweight by design.
3. **Only store notes in SQLite**: Rejected — notes should be repo-native files reviewable by humans, with SQLite as index.

## Implementation Plan

Phase 1: Define DurableInsightNote Rust domain type and schema
Phase 2: Implement note storage (markdown+frontmatter) and SQLite indexing
Phase 3: Implement scope-based fetch with tracking
Phase 4: Implement feedback scoring system
Phase 5: Implement prune candidate detection
Phase 6: Implement needs_human_review flagging
Phase 7: Implement archival logic
Phase 8: Integrate into task start (fetch) and task wrap-up (score/propose)
Phase 9: Add MCP and CLI tools

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- Notes can be created with full schema including scope
- Notes are fetched by scope at task start with tracking
- Feedback is recorded at task wrap-up
- Prune candidates are detected automatically based on usage patterns
- Notes needing human review are flagged with clear reasons
- Archived notes are preserved but excluded from active queries
- MCP and CLI tools expose all note operations
- SQLite queries for scope-based fetch complete in < 50ms for projects with up to 10,000 notes

## Risks / Dependencies

- Depends on SMET-I-0018 for base domain type patterns
- Depends on SMET-I-0031 for integration with execution records (notes fetched are recorded)
- Pruning thresholds need tuning based on real-world usage
- Must coordinate with SMET-I-0009 and SMET-I-0010 for MCP/CLI tools
- Risk of notes becoming stale if pruning is too conservative

## Suggested Tasks for Decomposition

1. Define DurableInsightNote Rust domain type
2. Implement note serialization (markdown+frontmatter round-trip)
3. Implement SQLite schema and scope-based indexing
4. Implement scope-based fetch with tracking
5. Implement feedback scoring (helpful/meh/harmful)
6. Implement prune candidate detection logic
7. Implement needs_human_review flagging logic
8. Implement archival logic
9. Integrate note fetch into task start workflow
10. Integrate note scoring/proposal into task wrap-up workflow
11. Add MCP tools for note CRUD, fetch, score
12. Add CLI tools for note inspection and management