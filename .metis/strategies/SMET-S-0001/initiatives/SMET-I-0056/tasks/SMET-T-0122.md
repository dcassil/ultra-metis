---
id: add-validate-command-with
level: task
title: "Add validate Command with DocumentValidator"
short_code: "SMET-T-0122"
created_at: 2026-03-18T04:31:30.673558+00:00
updated_at: 2026-03-18T04:38:45.226381+00:00
parent: SMET-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0056
---

# Add validate Command with DocumentValidator

## Parent Initiative

[[SMET-I-0056]] - CLI Architecture: Add Missing Commands and Parameter Parity

## Objective

Add a `validate` CLI command that checks document integrity — frontmatter correctness, phase validity, and parent cross-references. Supports validating a single document or all documents in the project.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cadre validate <short-code>` validates a single document
- [ ] `cadre validate --all` validates all documents in the project
- [ ] Checks: frontmatter parses correctly (required fields present)
- [ ] Checks: phase tag is valid for the document type
- [ ] Checks: parent_id references an existing document of valid type
- [ ] Checks: hierarchy rules (task has valid parent type, vision has no parent)
- [ ] Output: formatted table with Status (OK/WARN/ERROR) | Short Code | Issue | Severity
- [ ] Returns exit code 0 if all pass, 1 if any errors found
- [ ] Add `validate_document` and `validate_all` methods to DocumentStore
- [ ] Unit tests for each validation check

## Implementation Notes

### Technical Approach
1. Add `validate_document(short_code)` to DocumentStore returning `Vec<ValidationIssue>`
2. Add `validate_all()` that iterates all documents
3. ValidationIssue struct: `{ short_code, message, severity: Error|Warning }`
4. Checks to implement:
   - Frontmatter parses without error (already done by parse_document)
   - Phase tag present and valid for document type
   - parent_id exists and type is valid per HierarchyValidator
   - No circular parent references
5. Add `Validate` variant to CLI Commands enum
6. Formatted table output to stdout

### Key Files
- `crates/cadre-store/src/store.rs` — add validate methods
- `crates/cadre-cli/src/main.rs` — add Validate command

## Status Updates

*To be added during implementation*