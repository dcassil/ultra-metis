---
id: enrich-initialize-project-with
level: task
title: "Enrich initialize_project with Brownfield Detection and Bootstrap Analysis"
short_code: "SMET-T-0191"
created_at: 2026-03-27T15:52:58.795465+00:00
updated_at: 2026-03-27T16:01:55.723886+00:00
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

# Enrich initialize_project with Brownfield Detection and Bootstrap Analysis

## Parent Initiative

[[SMET-I-0093]]

## Objective

Modify the existing `initialize_project` MCP tool so that after creating the `.cadre/` directory, it detects whether the target is a brownfield project (has existing source files) and, if so, runs `BootstrapFlow::analyze()` and appends the results to the response. For greenfield projects, append a "Suggested Next Steps" section recommending ProductDoc creation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `initialize_project` detects brownfield vs greenfield after creating `.cadre/`
- [ ] For brownfield: response includes bootstrap analysis (languages, project type, tools, monorepo info)
- [ ] For greenfield: response includes suggestion to create a ProductDoc
- [ ] Response includes a "Suggested Next Steps" section with actionable guidance
- [ ] Existing behavior (creating `.cadre/` structure) is unchanged
- [ ] `cargo build` succeeds

## Implementation Notes

### Technical Approach

Modify `crates/cadre-mcp/src/tools/initialize_project.rs`:

1. After the existing `store.initialize(&self.prefix)?` call, scan the project directory for source files
2. Use a simple heuristic: if any non-hidden, non-`.cadre` files exist → brownfield
3. If brownfield, collect file paths and call `BootstrapFlow::analyze(&file_paths)`
4. Format the `BootstrapResult` as additional markdown sections appended to the existing response
5. If greenfield, append a simple "Suggested Next Steps" section

### Response Format (Brownfield)

```markdown
## Project Initialized

| Field | Value |
| ----- | ----- |
| Path  | /path/to/project |
| Prefix | CLI |
| Docs Dir | /path/to/project/.cadre/docs/ |
| Classification | Brownfield |

## Bootstrap Analysis

| Field | Value |
| ----- | ----- |
| Project Type | CliTool |
| Languages | Rust |
| Build Tools | Cargo |
| Monorepo | No |

### Detected Dev Tools
- clippy (linter)
- rustfmt (formatter)

### Suggestions
- Create a ProductDoc to capture product vision
- Architecture catalog has matching patterns for Rust CLI tools
```

### Response Format (Greenfield)

```markdown
## Project Initialized

| Field | Value |
| ----- | ----- |
| Path  | /path/to/project |
| Prefix | NEW |
| Docs Dir | /path/to/project/.cadre/docs/ |
| Classification | Greenfield |

## Suggested Next Steps
1. Create a ProductDoc to define the product vision
2. Browse the architecture catalog to select a reference architecture
3. Create your first Epic to begin planning work
```

### Extract shared formatting

The bootstrap result formatting logic should be extracted into a helper function in `tools/helpers.rs` so it can be reused by both `analyze_project` (T-0190) and `initialize_project`. Name it `format_bootstrap_result(result: &BootstrapResult) -> String`.

### Files to Modify
- **Modify**: `crates/cadre-mcp/src/tools/initialize_project.rs` (add brownfield detection + bootstrap call)
- **Modify**: `crates/cadre-mcp/src/tools/helpers.rs` (add `format_bootstrap_result` helper)

### Dependencies
- Shared formatting helper should be created first or concurrently with T-0190

## Status Updates

*To be added during implementation*