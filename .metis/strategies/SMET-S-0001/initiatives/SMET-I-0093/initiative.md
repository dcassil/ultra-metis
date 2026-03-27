---
id: expose-bootstrapflow-via-mcp-and
level: initiative
title: "Expose BootstrapFlow via MCP and Enrich initialize_project Response"
short_code: "SMET-I-0093"
created_at: 2026-03-27T15:46:34.712615+00:00
updated_at: 2026-03-27T16:05:06.238893+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: expose-bootstrapflow-via-mcp-and
---

# Expose BootstrapFlow via MCP and Enrich initialize_project Response

## Context

`cadre-core` contains a fully implemented `BootstrapFlow` system (`crates/cadre-core/src/domain/bootstrap/init_flow.rs`) that can:
- Scan repositories for languages and build tools via `RepoScanner`
- Detect monorepo patterns via `MonorepoDetector`
- Infer project type (Server, WebApp, CliTool, Library, etc.)
- Detect dev tooling (linters, formatters, test runners, CI) via `ToolDetector`
- Generate a `BootstrapSummary` with language facts, tool facts, and suggestions

**This code is completely orphaned.** It is not exposed through the MCP server or CLI. The current `initialize_project` MCP tool only creates the `.cadre/` directory structure and config file, returning nothing about the project it was just initialized in.

For brownfield (existing) codebases, this means the agent has zero context about what it just initialized and must manually discover everything the bootstrap system already knows how to detect.

## Goals & Non-Goals

**Goals:**
- Expose `BootstrapFlow` analysis as an MCP tool so agents can call it after (or during) initialization
- Enrich the `initialize_project` response to include bootstrap analysis results when the target is an existing codebase
- Return detected language, project type, build tools, linters, test runners, and monorepo structure
- Provide actionable suggestions in the response (e.g., "ProductDoc recommended", "Architecture catalog has matching patterns")

**Non-Goals:**
- Automatically creating documents (ProductDoc, ReferenceArchitecture) — that belongs to SMET-I-0094
- Plugin/skill layer changes — that belongs to SMET-I-0094
- Modifying the bootstrap analysis logic itself (it's already well-implemented)

## Architecture

### Overview

Two changes to `cadre-mcp`:

1. **New MCP tool: `analyze_project`** — Standalone tool that runs `BootstrapFlow` on a given path and returns the `BootstrapSummary` as structured output. Can be called independently of initialization.

2. **Enriched `initialize_project` response** — After creating the `.cadre/` directory, detect whether the project is brownfield (has existing source files). If so, automatically run bootstrap analysis and include results in the response.

### Data Flow

```
initialize_project(path, prefix)
  ├── create .cadre/ structure (existing behavior)
  ├── detect brownfield vs greenfield
  │   ├── greenfield: return as-is with suggestion to create ProductDoc
  │   └── brownfield: run BootstrapFlow
  │       ├── RepoScanner → languages, build tools
  │       ├── MonorepoDetector → workspace structure
  │       ├── ToolDetector → linters, formatters, CI
  │       └── ProjectType inference
  └── return enriched response with bootstrap summary
```

## Detailed Design

### 1. Wire BootstrapFlow into cadre-mcp

The `BootstrapFlow` types are in `cadre-core::domain::bootstrap`. The MCP server needs to:
- Import the bootstrap module
- Create an `analyze_project` tool handler that instantiates `BootstrapFlow` and runs it
- Format the `BootstrapSummary` as a markdown table in the MCP response

### 2. Enrich initialize_project

Modify `crates/cadre-mcp/src/tools/initialize_project.rs`:
- After successful `.cadre/` creation, scan for existing source files
- If files exist (brownfield), run `BootstrapFlow::analyze(path)`
- Append bootstrap results to the response table
- Include a "Suggested Next Steps" section based on findings

### 3. Response Format

The enriched response should include:
- **Project Classification**: greenfield | brownfield
- **Detected Languages**: with confidence levels
- **Project Type**: Server, WebApp, CliTool, Library, etc.
- **Build Tools**: cargo, npm, gradle, etc.
- **Dev Tools**: linters, formatters, test runners
- **Monorepo**: yes/no, detected packages
- **Suggested Next Steps**: what the agent should do next

## Alternatives Considered

1. **Merge bootstrap into initialize_project only** — Rejected because agents may want to re-analyze a project without re-initializing. A standalone `analyze_project` tool is more flexible.

2. **Expose via CLI only, not MCP** — Rejected because the primary consumer is the AI agent via MCP, not human CLI users.

3. **Return raw BootstrapSummary struct** — Rejected in favor of formatted markdown that agents can read and present to users directly.

## Implementation Plan

1. Create `analyze_project` MCP tool that wraps `BootstrapFlow`
2. Modify `initialize_project` to detect brownfield and run analysis
3. Format bootstrap results as structured markdown in both tool responses
4. Add unit tests for the new tool and enriched responses
5. Update MCP system prompt to document the new tool and enriched init flow