---
id: create-analyze-project-mcp-tool
level: task
title: "Create analyze_project MCP Tool Wrapping BootstrapFlow"
short_code: "SMET-T-0190"
created_at: 2026-03-27T15:52:57.982076+00:00
updated_at: 2026-03-27T16:01:54.839876+00:00
parent: SMET-I-0093
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0093
---

# Create analyze_project MCP Tool Wrapping BootstrapFlow

## Parent Initiative

[[SMET-I-0093]]

## Objective

Create a new `analyze_project` MCP tool in `cadre-mcp` that exposes the existing `BootstrapFlow` analysis from `cadre-core`. This tool accepts a project path and returns a formatted markdown summary of the project's detected languages, project type, build tools, dev tooling, and monorepo structure.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New file `crates/cadre-mcp/src/tools/analyze_project.rs` implementing `AnalyzeProjectTool`
- [ ] Tool accepts `project_path: String` parameter
- [ ] Tool calls `BootstrapFlow::analyze()` with file paths collected from the project directory
- [ ] Response includes: project classification (greenfield/brownfield), detected languages, inferred project type, build tools, dev tools (linters/formatters/test runners), monorepo info
- [ ] Tool registered in `mod.rs`, `all_tools.rs`, and `server.rs` dispatch macro
- [ ] `cargo build` succeeds with the new tool

## Implementation Notes

### Technical Approach

1. Create `crates/cadre-mcp/src/tools/analyze_project.rs`:
   - Define `AnalyzeProjectTool` struct with `#[mcp_tool(name = "analyze_project", description = "...")]`
   - Derive `Serialize, Deserialize, JsonSchema`
   - In `call_tool()`: collect file paths from `project_path` (excluding `.cadre/`, `target/`, `node_modules/`, `.*`), call `BootstrapFlow::analyze(&file_paths)`, format `BootstrapResult` as markdown

2. Format the response as a markdown table with sections:
   - Project Classification table (greenfield/brownfield, project type)
   - Detected Languages table (language, confidence)
   - Build & Dev Tools table (tool name, category)
   - Monorepo Info (if detected)
   - Suggestions list from `BootstrapSummary.suggestions`

3. Register the tool:
   - Add `pub mod analyze_project;` and `pub use analyze_project::*;` to `tools/mod.rs`
   - Add `AnalyzeProjectTool` to the `tool_box!` macro in `all_tools.rs`
   - Add `"analyze_project" => AnalyzeProjectTool` to `dispatch_tool!` in `server.rs`

### Key Types (from cadre-core)
- `BootstrapFlow::analyze(file_paths: &[String]) -> BootstrapResult`
- `BootstrapResult { scan, monorepo, tools, project_type, is_brownfield, summary }`
- `InferredProjectType` enum: Server, WebApp, CliTool, Library, etc.
- `BootstrapSummary { description, facts, suggestions }`

### Files to Create/Modify
- **Create**: `crates/cadre-mcp/src/tools/analyze_project.rs`
- **Modify**: `crates/cadre-mcp/src/tools/mod.rs` (add module + re-export)
- **Modify**: `crates/cadre-mcp/src/tools/all_tools.rs` (add to tool_box!)
- **Modify**: `crates/cadre-mcp/src/server.rs` (add to dispatch_tool!)

### Dependencies
- None — `BootstrapFlow` and all its types are already exported from `cadre-core`

## Status Updates

*To be added during implementation*