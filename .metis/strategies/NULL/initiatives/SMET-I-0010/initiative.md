---
id: extend-cli-support-for-the
level: initiative
title: "Extend CLI Support for the Stronger Engineering Model"
short_code: "SMET-I-0010"
created_at: 2026-03-11T19:59:55.460665+00:00
updated_at: 2026-03-11T19:59:55.460665+00:00
parent: SMET-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: extend-cli-support-for-the
---

# Extend CLI Support for the Stronger Engineering Model

## Context

The CLI is the primary human interface for Super-Metis. Metis already has CLI commands for basic document operations. Super-Metis needs to extend the CLI to support all new document types, quality operations, rule management, work leasing, and traceability — while keeping the command surface intuitive and discoverable.

## Governing Commitments

This initiative directly serves:
- **All durable project memory lives in the repo.** The CLI is the human's primary interface for creating, querying, and managing all repo-native artifacts — planning, architecture, rules, quality, and governance.
- **Structural guidance over improvisation** (Principle #3). CLI commands expose the same governance controls as MCP tools: quality subcommands surface gate status and baseline comparisons, rule subcommands enforce protected change workflows, lease subcommands make ownership explicit. Help text and examples make these operations discoverable.
- **Quality includes architectural integrity and is tracked over time.** Quality subcommands expose baseline capture, comparison, and gate checking — making quality history visible and actionable from the command line.
- **Governance and quality semantics remain consistent across execution modes** (Vision #9). CLI commands operate on the same persisted artifacts and enforce the same transition rules, quality gates, and protection semantics as MCP tools — humans and agents interact with one governance model.

## Goals & Non-Goals

**Goals:**
- Extend existing CLI commands to handle all new document types
- Add CLI commands for quality operations (capture baseline, compare, check gates)
- Add CLI commands for rule management and rule change proposals
- Add CLI commands for work leasing operations
- Add traceability and cross-reference query commands
- Improve CLI help and discoverability for the expanded command set
- Support both interactive and scriptable (non-interactive) modes

**Non-Goals:**
- Building a TUI (text-based UI) — the CLI is command-based, not interactive
- Replacing MCP as the agent interface — CLI is for humans
- Building shell completions for every shell (start with bash/zsh)

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