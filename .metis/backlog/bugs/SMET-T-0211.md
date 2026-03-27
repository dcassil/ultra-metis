---
id: fix-capture-quality-baseline
level: task
title: "Fix capture_quality_baseline directory resolution in cadre-mcp"
short_code: "SMET-T-0211"
created_at: 2026-03-27T18:20:33.039714+00:00
updated_at: 2026-03-27T18:59:47.395619+00:00
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

# Fix capture_quality_baseline directory resolution in cadre-mcp

## Objective

Fix `capture_quality_baseline` (and related tools like `index_code`) in `cadre-mcp` so they correctly resolve the project source directory when running against a `.cadre` project path. Currently fails with "No such file or directory" because it can't find the source files relative to the `.cadre` folder.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [ ] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users running `capture_quality_baseline` or `index_code` via MCP
- **Reproduction Steps**: 
  1. Initialize a Cadre project in a subdirectory (e.g., `crates/cadre-cli`)
  2. Call `capture_quality_baseline` with `project_path` pointing to the `.cadre` folder
  3. Tool fails with "No such file or directory (os error 2)"
- **Expected vs Actual**: Should resolve source files relative to the project root (parent of `.cadre/`). Instead fails because it can't find the directory to scan.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `capture_quality_baseline` resolves the project source directory correctly when given a `.cadre` path
- [ ] First tries `src/` relative to the project root (parent of `.cadre/`)
- [ ] If `src/` doesn't exist, falls back to asking the agent to help identify the correct source directory
- [ ] Stores the resolved source directory in `code-index.json` so subsequent calls use the cached path
- [ ] `index_code` also uses this same resolution logic
- [ ] Works for both top-level and nested `.cadre` projects (e.g., workspace crates)

## Implementation Notes

### Technical Approach
1. In the MCP tool handlers for `capture_quality_baseline` and `index_code`, add directory resolution logic:
   - Strip `.cadre` from `project_path` to get project root
   - Check if `code-index.json` already has a `source_dir` field — use it if present
   - Otherwise try `{project_root}/src/`
   - If that doesn't exist, return an error message asking the agent to provide the source directory, then store it
2. Update `code-index.json` schema to include a `source_dir` field
3. Once resolved and stored, all future calls use the cached directory

### Dependencies
- `crates/cadre-mcp/src/tools/` — MCP tool handlers
- `crates/cadre-core/` — may need updates to code indexing logic

## Status Updates

### 2026-03-27: Fix implemented

**Root causes identified:**
1. `index_code` tool used `project_path` (the `.cadre` folder) as `CodeIndexer` project root — glob patterns resolved against `.cadre/` instead of project root
2. `index_code` wrote `code-index.json` to `project_path/.cadre/code-index.json` creating double-nesting (`.cadre/.cadre/code-index.json`)
3. `capture_quality_baseline` wrote docs to `project_path/.cadre/docs/` creating the same double-nesting

**Changes made:**
- `crates/cadre-mcp/src/tools/helpers.rs`: Added `project_root_from()` and `cadre_internal_dir()` helpers that strip `.cadre` from project path to derive the actual project root
- `crates/cadre-mcp/src/tools/index_code.rs`: Uses `project_root_from()` for `CodeIndexer` project root; added `resolve_source_dir()` method that checks cached source_dir in index, falls back to `src/`, then asks agent
- `crates/cadre-mcp/src/tools/capture_quality_baseline.rs`: Uses `project_root_from()` for doc write path
- `crates/cadre-store/src/code_index.rs`: Added `source_dir: Option<String>` field to `CodeIndex` struct (serde-default, skip-if-none for backward compat)

**Tests:** All 8 code_index tests pass. Full workspace build succeeds. Pre-existing `test_pattern_matcher_ranking` failure unrelated (catalog entry count mismatch from Rust/Python additions).