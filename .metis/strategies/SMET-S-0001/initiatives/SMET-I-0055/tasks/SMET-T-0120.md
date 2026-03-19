---
id: implement-index-code-tool-with
level: task
title: "Implement index_code Tool with Tree-sitter"
short_code: "SMET-T-0120"
created_at: 2026-03-18T04:10:24.840719+00:00
updated_at: 2026-03-18T04:21:28.578205+00:00
parent: SMET-I-0055
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0055
---

# Implement index_code Tool with Tree-sitter

## Parent Initiative

[[SMET-I-0055]] - Tool Integration: Add Missing Tools and Parameter Support

## Objective

Implement a new `index_code` MCP tool that uses tree-sitter to extract source code symbols (functions, types, structs, traits) and store them for cross-referencing with Metis documents. This enables AI agents to understand the relationship between planning documents and code.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] tree-sitter and tree-sitter-rust added as dependencies to ultra-metis-core or new crate
- [ ] Symbol extraction works for Rust: functions, structs, traits, enums, impl blocks, type aliases
- [ ] Extracted symbols stored in `.ultra-metis/code-index.json` (file-based, no SQLite)
- [ ] Each symbol record includes: name, kind, file_path, line_number, signature
- [ ] `index_code` tool accepts: project_path, patterns (glob list), languages (optional)
- [ ] Tool returns count of indexed files and symbols
- [ ] Query interface to search indexed symbols by name or kind
- [ ] Tool registered in MCP tool definitions
- [ ] Unit tests for Rust symbol extraction (functions, structs, traits, enums)
- [ ] Integration test for end-to-end index + query workflow
- [ ] JavaScript/Python parsers as optional stretch (Rust is primary)

## Implementation Notes

### Technical Approach
1. Add `tree-sitter` and `tree-sitter-rust` crates as dependencies
2. Create `CodeIndexer` struct that walks source files matching glob patterns
3. For each file, parse with tree-sitter and extract named symbols via tree queries
4. Rust query targets: `function_item`, `struct_item`, `trait_item`, `enum_item`, `impl_item`, `type_item`
5. Store index as JSON in `.ultra-metis/code-index.json`
6. Provide `search_symbols(name_pattern, kind_filter)` query method
7. Register as MCP tool with schema in tools.rs

### Key Files
- `crates/ultra-metis-core/src/code_index/` - New module for indexing logic
- `crates/ultra-metis-store/src/store.rs` - Store integration
- `crates/ultra-metis-mcp/src/tools.rs` - Tool registration
- `Cargo.toml` (workspace) - New dependencies

### Dependencies
- tree-sitter crate (~0.24)
- tree-sitter-rust crate

### Risk Considerations
- tree-sitter native compilation may increase build time
- Large codebases could produce large index files — consider lazy indexing or incremental updates

## Status Updates

*To be added during implementation*