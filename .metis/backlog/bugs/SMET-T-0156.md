---
id: fix-metis-transition-phase-path
level: task
title: "Fix Metis transition_phase path resolution for documents under NULL strategy directory"
short_code: "SMET-T-0156"
created_at: 2026-03-23T17:11:10.244310+00:00
updated_at: 2026-03-23T17:24:11.050355+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Fix Metis transition_phase path resolution for documents under NULL strategy directory

## Objective

Fix `transition_phase` (and potentially other mutation tools) failing for documents stored under the `NULL/` strategy directory when their frontmatter `parent` field references a named strategy.

## Bug Details

### Type
- [x] Bug

### Priority
- [x] P2 - Medium (workaround: edit document content directly)

### Symptom
```
transition_phase SMET-I-0010 → "Not found: Document with short code 'SMET-I-0010' 
not found at path: .metis/strategies/SMET-S-0001/initiatives/SMET-I-0010/initiative.md"
```

But the document actually exists at: `.metis/strategies/NULL/initiatives/SMET-I-0010/initiative.md`

### Root Cause

The old Metis store uses **hierarchical directory-based path resolution** that constructs file paths from the document's `parent` field in frontmatter. When `transition_phase` needs to write back to the file, it constructs the expected path as:
```
.metis/strategies/{parent}/initiatives/{short_code}/initiative.md
```

But SMET-I-0010's frontmatter says `parent: SMET-S-0001` while the file is physically under `.metis/strategies/NULL/`. The constructed path doesn't match the actual path.

### Why read_document and search_documents work
These tools use `walkdir` directory scanning or a search index that finds files by content/short_code regardless of directory structure. They don't rely on parent-based path construction.

### Why transition_phase fails  
It needs to read AND write the document. The write path is constructed from frontmatter metadata, which points to the wrong directory.

### How the document ended up under NULL/
The document was likely created before being assigned to strategy SMET-S-0001, or was created with no parent and later had `parent: SMET-S-0001` added to its frontmatter without physically moving the file.

### Key Files
- `crates/ultra-metis-store/src/store.rs` — path resolution logic, `doc_path()` at line 262
- The old Metis path resolution (hierarchical) vs new ultra-metis (flat `docs/` dir)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `transition_phase` works for documents regardless of which strategy subdirectory they're physically in
- [ ] Path resolution uses directory scanning or an index rather than constructing paths from frontmatter parent fields
- [ ] Or: migrate orphaned documents so physical path matches frontmatter parent

## Implementation Notes

### Options
1. **Index-based resolution**: Build a short_code → file_path index at startup, use it for all operations
2. **Walkdir-based resolution**: Find documents by scanning directories (like read_document already does)  
3. **Data migration**: Move `.metis/strategies/NULL/initiatives/SMET-I-0010/` to `.metis/strategies/SMET-S-0001/initiatives/SMET-I-0010/`
4. **Hybrid**: Fix the path construction to fall back to directory scanning when the constructed path doesn't exist

Option 1 (index) is the most robust long-term fix. Option 3 is the quickest workaround.

## Status Updates

*Discovered 2026-03-23 when attempting to transition SMET-I-0010 after completing scoped CLI work.*

### 2026-03-23 — Fixed via data migration

**Root cause**: SMET-I-0010 (and potentially other initiatives created early) were physically stored under `strategies/NULL/` but their frontmatter `parent` field referenced `SMET-S-0001`. The installed Metis binary (v1.1.0) uses the DB's `filepath` column to resolve documents, and the DB gets synced from the filesystem. The path mismatch meant `transition_phase` constructed a wrong path.

**Investigation**: The `reference - original metis/` codebase has an unreleased v1→v2 migration that flattens the `strategies/` layout. But the installed binary (v1.1.0) still expects `strategies/`. Attempting the migration broke everything.

**Fix applied**: Moved all initiatives to match their frontmatter `parent` field:
- Initiatives with `parent: SMET-S-0001` → `strategies/SMET-S-0001/initiatives/`
- Initiatives with `parent: NULL` or missing → `strategies/NULL/initiatives/`
- Same for archived initiatives
- DB resync happened automatically via `prepare_workspace`

**Verified**: `transition_phase SMET-I-0010 design` succeeds after migration.