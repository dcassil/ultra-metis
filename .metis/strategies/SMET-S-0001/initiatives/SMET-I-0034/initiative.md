---
id: local-installation-and-end-to-end
level: initiative
title: "Local Installation and End-to-End Working Setup"
short_code: "SMET-I-0034"
created_at: 2026-03-17T02:59:10.644185+00:00
updated_at: 2026-03-17T18:28:46.792373+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: ultra-metis-core-engine-repo
initiative_id: local-installation-and-end-to-end
---

# Local Installation and End-to-End Working Setup Initiative

## Context

Ultra-metis has a complete core domain library (736+ tests, 12 domain modules), but it's not yet usable as an installed tool. To replace or coexist with the existing metis plugin, ultra-metis needs:
- An MCP server that exposes core functionality as tools (like the existing metis MCP server)
- A CLI binary for direct command-line usage
- Plugin manifest and configuration so Claude Code can discover and load it
- End-to-end verification that document CRUD, phase transitions, and all core workflows actually work through the MCP/CLI interface

This initiative depends on SMET-I-0033 (namespace rename) being completed first so the plugin registers under the correct name.

## Goals & Non-Goals

**Goals:**
- Build an MCP server binary crate (`ultra-metis-mcp`) that exposes core operations as MCP tools
- Implement MCP tools matching the existing metis plugin surface: initialize_project, list_documents, read_document, create_document, edit_document, transition_phase, search_documents, archive_document, reassign_parent
- Build a CLI binary crate (`ultra-metis-cli`) for command-line access to the same operations
- Create a Claude Code plugin manifest (`plugin.json`) so it can be installed as a plugin
- Create `.mcp.json` configuration for MCP server registration
- End-to-end test: initialize a project, create vision → initiative → task, transition phases, verify file-based persistence
- Document installation steps

**Non-Goals:**
- Feature parity with every edge case of the existing metis plugin (MVP tool set first)
- GUI or web interface
- SQLite indexing layer (file-based persistence first, SQLite can come later)
- Publishing to any package registry

## Detailed Design

### Crate Structure
```
ultra-metis/
  Cargo.toml (workspace)
  crates/
    ultra-metis-core/    (existing — domain types and logic)
    ultra-metis-mcp/     (new — MCP server binary)
    ultra-metis-cli/     (new — CLI binary)
```

### MCP Server (`ultra-metis-mcp`)
- Uses `rmcp` or equivalent Rust MCP SDK
- Exposes each operation as an MCP tool with JSON schema parameter validation
- File-based persistence: reads/writes markdown+frontmatter documents in `.ultra-metis/` directory
- Runs as stdio transport (Claude Code spawns it)

### CLI (`ultra-metis-cli`)
- `ultra-metis init` — initialize a project
- `ultra-metis list` — list documents
- `ultra-metis read <short-code>` — read a document
- `ultra-metis create <type> <title>` — create a document
- `ultra-metis transition <short-code>` — advance phase
- `ultra-metis edit <short-code>` — edit document content
- Uses `clap` for argument parsing

### Plugin Manifest
- `plugin.json` with MCP server configuration
- `.mcp.json` for Claude Code auto-discovery
- Registers as `ultra-metis` (not `metis`) to avoid conflicts

### File Persistence Layer
- Documents stored as markdown+frontmatter in `.ultra-metis/` project directory
- Short code counter in `.ultra-metis/config.toml`
- Directory structure mirrors existing metis convention but under `.ultra-metis/`

## Alternatives Considered

1. **Reuse existing metis MCP server code**: Rejected — existing metis is TypeScript, we're building in Rust for the core type safety guarantees.
2. **Skip CLI, MCP only**: Rejected — CLI is useful for debugging and scripting.
3. **Use `.metis/` directory**: Rejected — would conflict with the existing metis plugin.

## Implementation Plan

Phase 1: Build file persistence layer (read/write documents to `.ultra-metis/`)
Phase 2: Build MCP server with core tools
Phase 3: Build CLI with core commands
Phase 4: Create plugin manifest and installation config
Phase 5: End-to-end integration test