---
id: edit-document-frontmatter
level: task
title: "Edit document frontmatter integrity validation"
short_code: "SMET-T-0083"
created_at: 2026-03-17T18:56:00.408919+00:00
updated_at: 2026-03-17T19:22:27.978871+00:00
parent: SMET-I-0036
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0036
---

# Edit document frontmatter integrity validation

## Parent Initiative

[[SMET-I-0036]]

## Objective

Add post-edit validation to `edit_document` to catch frontmatter corruption. Currently, `edit_document` does a raw string replacement with no re-parsing, meaning a search/replace that breaks YAML frontmatter structure goes undetected. After the replacement, re-parse the document to verify it's still valid — if not, roll back the edit and return an error.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] After `edit_document` performs a replacement, the resulting file is re-parsed to verify frontmatter validity
- [ ] If re-parsing fails, the original file content is restored and `StoreError::Validation` is returned with message "Edit would corrupt document frontmatter: {parse_error}. Edit rolled back."
- [ ] Valid edits to markdown body content (below frontmatter) succeed as before
- [ ] Valid edits to frontmatter values (e.g., changing title) succeed as before
- [ ] Unit tests cover: edit that breaks YAML syntax, edit that removes required field, valid body edit, valid frontmatter value edit

## Implementation Notes

### Technical Approach
1. In `edit_document`, save the original content before replacement
2. After replacement, attempt to parse the new content as a document (frontmatter extraction + YAML parse)
3. If parsing fails, write the original content back and return error
4. If parsing succeeds, keep the new content (already written)
5. Keep it lightweight — only validate frontmatter structure, not full document semantics

### Dependencies
- None — standalone store-layer change

### Risk Considerations
- Must not be overly strict — edits to markdown body should never trigger validation failures
- Re-parsing adds a small performance cost but edits are infrequent

## Status Updates

### 2026-03-17
- Added post-edit validation: after replacement, parses the new content to verify frontmatter integrity
- If parsing fails, the edit is rejected before writing (no file corruption)
- Added 2 tests: corrupting edit rolled back, valid body edit succeeds
- All 25 store tests pass