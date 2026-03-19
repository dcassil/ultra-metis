---
id: cli-architecture-add-missing
level: initiative
title: "CLI Architecture: Add Missing Commands and Parameter Parity"
short_code: "SMET-I-0056"
created_at: 2026-03-17T22:43:37.218486+00:00
updated_at: 2026-03-18T04:38:49.440232+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: cli-architecture-add-missing
---

# CLI Architecture: Add Missing Commands and Parameter Parity

## Context

Ultra-Metis CLI has 8 subcommands (init, list, read, create, edit, transition, search, archive). The MCP layer now has 10 tools including `reassign_parent` and `index_code` (added in SMET-I-0055), plus enhanced parameters (replace_all, force, document_type filter, limit, include_archived). The CLI needs to catch up with:

1. **Missing commands** — `validate`, `status`, and `reassign` have no CLI equivalents
2. **Parameter gaps** — MCP has replace_all, force, type/limit filters but CLI doesn't expose them
3. **No validation tooling** — no way to check document integrity from the command line

Previously scoped items removed:
- `sync` — deferred to SMET-I-0057 (SQLite FTS), no database index to sync with yet
- `config` — no flight-level config backing store exists; aspirational
- Interactive REPL — separate initiative, large scope (rustyline, autocomplete)

## Goals & Non-Goals

**Goals:**
- Add `validate` command for document integrity checking
- Add `status` command for work dashboard
- Add `reassign` command to mirror MCP reassign_parent tool
- Add missing parameters to existing commands (--replace-all, --force, --type, --limit, --include-archived)
- Use user_message() for all error output (consistent with MCP)

**Non-Goals:**
- Interactive REPL mode (separate initiative)
- sync command (depends on SMET-I-0057)
- config command (no backing store)
- Shell completions
- Quality/rules/lease commands (SMET-I-0009/I-0010 scope)

## Requirements

### New Commands

**validate <short-code|--all>**
- REQ-001: Validate document frontmatter (required fields, type correctness)
- REQ-002: Validate phase is valid for document type
- REQ-003: Check parent_id cross-references (points to existing document of valid type)
- REQ-004: Support --all flag to validate entire project
- REQ-005: Output table: Status | Short Code | Issue | Severity

**status**
- REQ-006: Show active initiatives with phase and child task counts
- REQ-007: Show active tasks grouped by parent
- REQ-008: Show summary counts by phase (todo/active/completed)

**reassign <short-code> [--parent <id>] [--backlog <category>]**
- REQ-009: Move task to different parent initiative
- REQ-010: Move task to backlog with category
- REQ-011: Validate same rules as MCP tool (task only, parent phase check)

### Parameter Parity with MCP
- REQ-012: `edit --replace-all` flag
- REQ-013: `transition --force` flag
- REQ-014: `search --type <type>` filter
- REQ-015: `search --limit <n>` cap
- REQ-016: `search --include-archived` flag

## Testing Strategy

### Unit Tests
- Validate command: frontmatter checks, cross-reference validation, phase validation
- Status aggregation: phase counting, grouping by parent

### Integration Tests
- validate --all on project with valid and invalid documents
- status with realistic document set
- reassign between initiatives and to backlog
- edit --replace-all, transition --force, search --type --limit

## Implementation Plan

### Task 1: Add validate command with DocumentValidator
### Task 2: Add status command with dashboard aggregation
### Task 3: Add reassign command mirroring MCP tool
### Task 4: Add missing parameters to existing CLI commands

### Exit Criteria
- validate catches broken parent refs, invalid phases, missing fields
- status shows useful dashboard of active work
- reassign works for initiative-to-initiative and to-backlog moves
- All new MCP parameters accessible from CLI
- Error messages use user_message() consistently
- All existing CLI tests still pass