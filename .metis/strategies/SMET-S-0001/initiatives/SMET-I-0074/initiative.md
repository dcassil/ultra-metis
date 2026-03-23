---
id: rename-cadre-to-cadre
level: initiative
title: "Rename Cadre to Cadre: Namespace, Binaries, Plugin, and Project Directory"
short_code: "SMET-I-0074"
created_at: 2026-03-23T17:28:01.410740+00:00
updated_at: 2026-03-23T20:24:53.936024+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: rename-cadre-to-cadre
---

# Rename Cadre to Cadre: Namespace, Binaries, Plugin, and Project Directory Initiative

## Context

ADR SMET-A-0001 decided to adopt superpowers execution patterns with Cadre as the persistent state backbone. "Cadre" stands for **Constrained AI Developer Really Awesome Engine**. Phase 0 of that ADR's implementation roadmap requires a complete namespace rename from "cadre" to "cadre" before any integration work begins. All subsequent initiatives (SMET-I-0075 through SMET-I-0078) build on the Cadre namespace and are blocked until this rename is complete.

The rename touches every layer: Rust crate names and imports, compiled binary names, Claude Code plugin directory and all commands/hooks/scripts/skills/agents, MCP server configuration, Makefile, and CLAUDE.md.

The existing `.metis/` folder in this repository is NOT renamed. `.cadre` applies only to newly initialized projects.

## Goals & Non-Goals

**Goals:**
- Rename all four Rust crates: cadre-core → cadre-core, cadre-store → cadre-store, cadre-mcp → cadre-mcp, cadre-cli → cadre-cli
- Rename crate directories under `crates/` to match
- Update root Cargo.toml workspace members and all internal dependencies
- Update all `use` statements, module paths, and string literals referencing old crate names
- Rename binaries: cadre-mcp → cadre-mcp, cadre → cadre
- Rename plugin directory: plugins/cadre/ → plugins/cadre/
- Rename all commands: /cadre-ralph → /cadre-ralph, etc.
- Update all MCP tool prefixes: mcp__cadre__ → mcp__cadre__
- Update .mcp.json, Makefile, CLAUDE.md
- Set default project directory for new projects to `.cadre/`
- Ensure `make build`, `make install`, `make test` all pass

**Non-Goals:**
- Renaming existing `.metis/` directory in this repo
- Migrating existing document data
- Adding new features or changing functionality
- Renaming the Git repository or changing GitHub remote URL

## Detailed Design

### Execution Order
1. Rename crate directories via `git mv`
2. Update all Cargo.toml files
3. Update all Rust source files (use statements, string literals)
4. Verify `cargo build` and `cargo test` pass
5. Rename plugin directory and files
6. Update all plugin file contents
7. Update .mcp.json, Makefile, CLAUDE.md
8. Update default project directory in init code
9. Final sweep: `rg "cadre"` to catch stragglers

## Alternatives Considered

1. **Gradual rename with compatibility aliases**: Rejected — pre-release project, clean cut is simpler
2. **Rename plugin only, keep Rust crate names**: Rejected — permanent confusion between crate and product names
3. **Defer rename until after integration work**: Rejected per ADR discussion decision #1

## Implementation Plan

Single-phase initiative with linear execution: Rust crates → plugin → config/docs → validation sweep.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- All four crates compile under new names
- `cadre-mcp` and `cadre` binaries produced by `make build`
- `make test` passes with zero failures
- All slash commands use `/cadre-*` naming
- All MCP tool references use `mcp__cadre__` prefix
- `rg "cadre"` finds zero hits outside archival directories

## Dependencies

- **Blocked by**: Nothing
- **Blocks**: SMET-I-0075, I-0076, I-0077, I-0078