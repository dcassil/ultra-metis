---
id: extend-cli-support-for-the
level: initiative
title: "Extend CLI Support for the Stronger Engineering Model"
short_code: "SMET-I-0010"
created_at: 2026-03-11T19:59:55.460665+00:00
updated_at: 2026-03-11T19:59:55.460665+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: ultra-metis-core-engine-repo
initiative_id: extend-cli-support-for-the
---

# Extend CLI: Expose Completed Domain Types for Human Use

## Strategy Update (2026-03-18)

**Revised approach**: Rescoped to mirror I-0009 changes. Priority is CLI commands humans actually use interactively. Skip lease commands, orchestration commands. Focus on making the built domain types queryable from the terminal.

**Key decisions:**
- Focus on read/query commands for quality, rules, notes, traceability, architecture
- Skip lease commands (POST-MVP)
- Skip execution/orchestration commands (plugin skills handle this, not CLI)
- Ensure parity with MCP tool additions from I-0009
- Add `--json` output for scripting, human-readable output by default
- CLI is for humans; MCP is for agents — design accordingly

## Context

Ultra-Metis CLI currently supports document CRUD, phase transitions, search, archive, reassign, validate, and status (completed in SMET-I-0056). The richer domain model (quality baselines, rules, notes, traceability, architecture catalog) is built in ultra-metis-core but not accessible from the command line. Humans need CLI access to inspect and manage these artifacts.

## Goals & Non-Goals

**Goals:**
- Add CLI commands for quality inspection (view baselines, compare, check gates)
- Add CLI commands for rule browsing (list rules by scope, view rule details)
- Add CLI commands for note management (list, view, score notes)
- Add CLI commands for traceability queries (trace ancestry/descendants)
- Add CLI commands for architecture catalog browsing
- Support `--json` output for all new commands
- Improve `--help` with usage examples

**Non-Goals:**
- Lease commands (POST-MVP)
- Orchestration commands (execution is plugin-level)
- TUI/interactive mode
- Shell completions (low priority)

## Detailed Design

### What to Reuse from `metis/`
- The existing CLI framework (likely clap-based in Rust)
- Command structure and argument patterns
- Output formatting conventions
- The existing command implementations as a base

### What to Change from `metis/`
- Extend `create` command to handle new document types
- Extend `list` and `search` commands with new type filters
- Update `transition` command for new phase flows
- Improve help text with examples for each command

### What is Net New
- `super-metis quality capture` — capture a quality baseline
- `super-metis quality compare` — compare two baselines
- `super-metis quality gate` — check quality gate status
- `super-metis quality validate` — record a validation result
- `super-metis rules list` — list active engineering rules by scope
- `super-metis rules propose` — create a rule change proposal
- `super-metis notes fetch` — fetch notes by scope (repo/package/subsystem/path/symbol)
- `super-metis notes create` — create a durable insight note
- `super-metis notes score` — record feedback on a note
- `super-metis notes inspect` — browse and filter notes
- `super-metis trace` — trace document ancestry/descendants and cross-references
- `super-metis execution list` — list execution records
- `super-metis execution show` — show execution record details
- `super-metis mode` — show or set autonomy mode (tight/mixed/autonomous)
- `super-metis workflow list` — list available workflow templates
- `super-metis init` — enhanced repo-aware initialization (SMET-I-0008)
- Improved `--help` with usage examples and workflow guides
- Post-MVP: `super-metis lease acquire/release` — work lease operations

## Alternatives Considered

1. **Wrap MCP tools as CLI**: Rejected because CLI needs different UX patterns (formatted output, interactive prompts) than MCP tools.
2. **Single monolithic command with subcommands for everything**: This is the approach — `super-metis` as the root command with organized subcommand groups.
3. **Separate CLIs for different concern areas**: Rejected because a single CLI is easier to discover and learn.

## Implementation Plan

Phase 1: Extend existing commands for new document types
Phase 2: Add quality subcommand group
Phase 3: Add rules subcommand group
Phase 4: Add lease subcommand group
Phase 5: Add trace command
Phase 6: Enhance init command with repo-awareness
Phase 7: Improve help text and add usage examples
Phase 8: Add bash/zsh shell completions

## Acceptance Criteria

- All new document types can be created, listed, searched, and transitioned via CLI
- Quality operations work from the CLI (capture, compare, gate check)
- Rule operations work from the CLI (list, propose, approve)
- Lease operations work from the CLI (acquire, release, status)
- Traceability queries produce clear, readable output
- Help text for every command includes at least one usage example
- CLI output is both human-readable and parseable (support `--json` flag)
- Exit codes are meaningful for scripting

## Risks / Dependencies

- Depends on all domain model work (SMET-I-0018, I-0019, I-0020, I-0004, I-0006, I-0007)
- Depends on new MVP initiatives: SMET-I-0029 (cognitive operations), SMET-I-0030 (notes), SMET-I-0031 (execution records), SMET-I-0032 (gates/autonomy)
- CLI surface grows significantly — risk of discoverability problems
- Must coordinate with SMET-I-0008 for init command enhancements
- Must maintain consistent UX patterns across all new commands
- Lease commands are post-MVP — only implement when SMET-I-0023 is active

## Codebase Areas to Inspect

- `metis/src/cli/` or `metis/src/commands/` — existing CLI commands
- `metis/src/main.rs` — command routing
- `metis/Cargo.toml` — CLI framework dependency (likely clap)
- `metis/src/output/` or equivalent — output formatting

## Suggested Tasks for Decomposition

1. Extend create/list/search/transition commands for new types
2. Implement `quality capture` command
3. Implement `quality compare` command
4. Implement `quality gate` command
5. Implement `rules` subcommand group
6. Implement `lease` subcommand group
7. Implement `trace` command
8. Add `--json` output support to all commands
9. Write comprehensive help text with examples
10. Add bash/zsh shell completions