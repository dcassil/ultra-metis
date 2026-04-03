---
id: persistence-search-parent-child
level: initiative
title: "Persistence & Search: Parent-Child Queries and Search Hardening"
short_code: "SMET-I-0057"
created_at: 2026-03-17T22:43:37.907549+00:00
updated_at: 2026-03-20T16:55:47.439745+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"
  - "#feature-search"
  - "#category-infrastructure"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: persistence-search-parent-child
---

# Persistence & Search: Parent-Child Queries and Search Hardening

## Status Update (2026-03-20)

**Heavily scaled down from original XL scope.** Most of the original scope is either implemented or unnecessary:

- `index_code` MCP tool exists with tree-sitter (Rust, JS, Python, Go) — stores to `.cadre/code-index.json`
- `search_documents` has all needed parameters (document_type, limit, include_archived)
- `reassign_parent` tool works for task reorganization
- Claude Code Grep/Glob handles codebase search — no need to duplicate
- SQLite FTS5 is over-engineering at current scale (< 500 docs)

Original scope included: full SQLite FTS5 migration, database-backed relationships, lazy migration strategy, code-to-doc cross-referencing, sync command, performance benchmarks for 10,000+ docs. All of this has been cut.

## Context

The immediate need is `find_children_by_type` — a query that discovers child documents under a parent by type. This is required by SMET-I-0068 (Architecture document type) and SMET-I-0069 (lifecycle hooks) to find Architecture documents linked to Stories and to walk from Tasks up to parent Stories.

Secondary: minor `index_code` hardening for edge cases on larger codebases.

## Goals & Non-Goals

**Goals:**
- Implement `find_children_by_type(parent_short_code, document_type)` query on document store — scans parent directory for child documents of specified type
- Add `--parent` filter to `list_documents` MCP tool and CLI command
- Harden `index_code` error handling for edge cases (missing files, unsupported syntax, very large repos)

**Non-Goals:**
- SQLite FTS5 migration
- Database-backed relationship schema
- Code-to-documentation cross-referencing
- Real-time incremental indexing
- Phrase queries, fuzzy matching, or ranking
- Sync command or lazy migration

## Detailed Design

### find_children_by_type Query

New method on the document store:

```rust
pub fn find_children_by_type(
    &self,
    parent_short_code: &str,
    document_type: DocumentType,
) -> Result<Vec<ShortCode>>
```

Implementation: locate the parent document's directory, scan subdirectories for documents matching the specified type by reading frontmatter. Returns short codes of matching children.

This is also exposed through the `HookContext` trait (SMET-I-0069) for use by transition hooks.

### list_documents --parent Filter

Add optional `parent_id` parameter to `list_documents` MCP tool and `--parent` flag to CLI. When provided, only returns documents that are direct children of the specified parent.

### index_code Hardening

- Graceful handling of files that fail to parse (skip with warning, don't abort)
- Timeout for individual file parsing (prevent hangs on malformed files)
- Better error messages when tree-sitter grammars are missing

## Implementation Plan

Phase 1: Implement `find_children_by_type` on document store
Phase 2: Add `--parent` / `parent_id` filter to list_documents (MCP + CLI)
Phase 3: Harden index_code error handling
Phase 4: Unit and integration tests

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `find_children_by_type` returns correct children for a given parent and type
- [ ] `list_documents` with parent_id filter works via MCP and CLI
- [ ] `index_code` gracefully handles parse failures without aborting
- [ ] Unit tests for child discovery, filtering, and error cases

## Risks / Dependencies

- SMET-I-0068 and SMET-I-0069 depend on `find_children_by_type` — this should be implemented first or concurrently with SMET-I-0068
- File-scan approach for child discovery is O(n) but acceptable at current document counts