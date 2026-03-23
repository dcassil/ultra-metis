---
id: add-missing-parameters-to-existing
level: task
title: "Add Missing Parameters to Existing CLI Commands"
short_code: "SMET-T-0125"
created_at: 2026-03-18T04:31:33.701496+00:00
updated_at: 2026-03-18T04:38:38.068706+00:00
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

# Add Missing Parameters to Existing CLI Commands

## Parent Initiative

[[SMET-I-0056]] - CLI Architecture: Add Missing Commands and Parameter Parity

## Objective

Update existing CLI commands to expose the new parameters added in SMET-I-0055: replace_all for edit, force for transition, and type/limit/include_archived for search. Also update error handling to use user_message() consistently.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `edit` command: add `--replace-all` flag, calls `edit_document_with_options`
- [ ] `transition` command: add `--force` flag, calls `transition_phase_with_options`
- [ ] `search` command: add `--type <type>` filter
- [ ] `search` command: add `--limit <n>` cap
- [ ] `search` command: add `--include-archived` flag
- [ ] All search params passed to `search_documents_with_options`
- [ ] All error handling uses `.map_err(|e| e.user_message())` instead of `.to_string()`
- [ ] Existing CLI behavior unchanged when new flags are omitted
- [ ] Help text updated for each modified command

## Implementation Notes

### Technical Approach
1. Add clap args to existing command variants:
   - `Edit`: add `#[arg(long)] replace_all: bool`
   - `Transition`: add `#[arg(long)] force: bool`
   - `Search`: add `#[arg(long, name = "type")] doc_type: Option<String>`, `#[arg(long)] limit: Option<usize>`, `#[arg(long)] include_archived: bool`
2. Update handler functions to pass new args to store methods
3. Change all `.map_err(|e| e.to_string())` to `.map_err(|e| e.user_message())`
4. Verify clap help output shows new options

### Key Files
- `crates/cadre-cli/src/main.rs` — all changes in one file

## Status Updates

*To be added during implementation*