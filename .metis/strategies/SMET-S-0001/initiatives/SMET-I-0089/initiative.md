---
id: cadre-plugin-shell-script-quality
level: initiative
title: "Cadre Plugin: Shell Script Quality and Linting"
short_code: "SMET-I-0089"
created_at: 2026-03-26T17:21:38.946298+00:00
updated_at: 2026-03-26T17:21:38.946298+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: cadre-plugin-shell-script-quality
---

# Cadre Plugin: Shell Script Quality and Linting Initiative

## Context

The Cadre plugin at `plugins/cadre/` contains **8 bash scripts totaling 812 lines** with no static analysis. These scripts are critical infrastructure - they orchestrate Claude Code's interaction with the Cadre system (session hooks, task execution setup, dependency checking). A bug in these scripts can silently break the entire workflow.

### Current Shell Script Inventory

| File | LOC | Purpose |
|------|-----|---------|
| `scripts/setup-cadre-ralph.sh` | 232 | Read document, map story type to skills, create ralph-loop state |
| `scripts/setup-cadre-decompose.sh` | 196 | Read epic, build decomposition prompt, create state file |
| `hooks/session-start-hook.sh` | 139 | Detect .metis directory, inject project context |
| `hooks/pre-compact-hook.sh` | 98 | Re-inject context after compaction |
| `scripts/check-dependencies.sh` | 70 | Verify cadre CLI, ralph-loop, superpowers |
| `hooks/subagent-start-hook.sh` | 57 | Minimal context for subagents |
| `hooks/block-todowrite.sh` | 20 | Block TodoWrite, redirect to MCP tools |

### Issues Found

1. **No shellcheck enforcement**: No `.shellcheckrc`, no CI step, no local tooling
2. **Scripts over 100 LOC**: `setup-cadre-ralph.sh` (232 LOC) and `setup-cadre-decompose.sh` (196 LOC) should be decomposed
3. **Inconsistent quoting**: Some variables unquoted in contexts that could break with spaces
4. **No `set -euo pipefail`**: Some scripts missing strict error handling
5. **Duplicated logic**: Both setup scripts share patterns for document reading, state file creation, and JSON escaping
6. **Hardcoded paths**: Plugin cache paths checked in multiple locations with duplicated logic
7. **No function decomposition**: Scripts are linear sequences rather than composed from reusable functions

### Additional Plugin Files

| File | LOC | Language | Issues |
|------|-----|---------|--------|
| `hooks/hooks.json` | 49 | JSON | No schema validation |
| `.claude-plugin/plugin.json` | 8 | JSON | Minimal - fine |
| `.mcp.json` | 9 | JSON | Fine |
| 5 SKILL.md files | 129-197 | Markdown | No markdownlint |
| 4 command .md files | 14-116 | Markdown | No markdownlint |
| 1 agent .md file | 197 | Markdown | No markdownlint |

## Goals & Non-Goals

**Goals:**
- Add `.shellcheckrc` at workspace root and fix all shellcheck warnings in plugin scripts
- Add `set -euo pipefail` to all scripts that don't already have it
- Decompose `setup-cadre-ralph.sh` (232 LOC) and `setup-cadre-decompose.sh` (196 LOC) into smaller functions or sourced files
- Extract shared shell utilities (JSON escaping, state file creation, document reading) into a common library script
- Add shellcheck to CI (shared with SMET-I-0084)
- Consistent quoting throughout all scripts

**Non-Goals:**
- Rewriting bash scripts in another language (Rust, Python)
- Adding markdown linting (low priority, cosmetic)
- Changing plugin architecture or hook design
- Adding JSON schema validation for hooks.json/plugin.json

## Detailed Design

### 1. Shared Shell Library

Create `plugins/cadre/scripts/lib/common.sh` (~80 LOC) containing reusable functions:

```bash
#!/usr/bin/env bash
# Common utilities for Cadre plugin scripts

# Read a document via cadre CLI and return content
read_document() {
    local short_code="$1"
    cadre read "$short_code" 2>/dev/null
}

# Escape string for JSON embedding
json_escape() {
    local input="$1"
    echo "$input" | sed 's/"/\\"/g; s/\t/\\t/g'
}

# Create ralph-loop state file
create_state_file() {
    local state_dir="$1"
    local prompt="$2"
    local max_iterations="${3:-20}"
    # ...
}

# Resolve plugin path from cache or vendor
resolve_plugin_path() {
    local plugin_name="$1"
    # Checks ~/.claude/plugin-cache, marketplace, vendored locations
}

# Validate short code format
validate_short_code() {
    local code="$1"
    [[ "$code" =~ ^[A-Z]+-[A-Z]-[0-9]+$ ]]
}
```

### 2. Script Decomposition

**setup-cadre-ralph.sh (232 -> ~120 LOC)**:
- Source `lib/common.sh` for shared utilities
- Extract story-type-to-skills mapping into a function
- Extract prompt building into a function
- Main script becomes: read doc -> map skills -> build prompt -> create state

**setup-cadre-decompose.sh (196 -> ~100 LOC)**:
- Source `lib/common.sh` for shared utilities
- Extract epic reading and metadata extraction into a function
- Main script becomes: read epic -> build decompose prompt -> create state

### 3. Shellcheck Compliance

Run shellcheck on all scripts and fix:
- **SC2086**: Double-quote variable expansions
- **SC2155**: Declare and assign separately
- **SC2034**: Unused variables
- **SC2164**: Use `cd ... || exit` in case cd fails
- **SC2068**: Quote array expansions

### 4. Strict Mode

Ensure all scripts start with:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

This catches:
- `-e`: Exit on error
- `-u`: Treat unset variables as errors
- `-o pipefail`: Fail on pipe errors (not just last command)

### 5. Hook Script Improvements

The hook scripts (`session-start-hook.sh`, `pre-compact-hook.sh`, `subagent-start-hook.sh`) are output-only (they echo JSON/text to stdout). They're simpler but should still:
- Use strict mode
- Quote all variables
- Pass shellcheck cleanly

## Alternatives Considered

1. **Rewrite in Rust**: Would eliminate shell issues entirely but adds compilation step for plugin hooks. Hooks need to be fast shell scripts. Not practical.
2. **Rewrite in Python**: Adds a runtime dependency. Bash is the right choice for Claude Code plugin hooks.
3. **Use bats for testing**: Good idea for the future but out of scope. This initiative focuses on linting and structure.

## Implementation Plan

1. **Add .shellcheckrc and run baseline** (1 task): Create config, run shellcheck, catalog all warnings
2. **Fix shellcheck warnings** (1 task): Fix quoting, strict mode, and other warnings across all scripts
3. **Extract shared library** (1 task): Create lib/common.sh with shared functions, refactor setup scripts to source it
4. **Decompose large scripts** (1 task): Break setup-cadre-ralph.sh and setup-cadre-decompose.sh into functions
5. **Verification** (part of each task): Run shellcheck, test plugin workflows end-to-end