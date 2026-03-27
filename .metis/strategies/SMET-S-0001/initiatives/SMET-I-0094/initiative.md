---
id: post-initialization-guided-setup
level: initiative
title: "Post-Initialization Guided Setup: Brownfield Analysis, ProductDoc, and Architecture Selection"
short_code: "SMET-I-0094"
created_at: 2026-03-27T15:46:36.234282+00:00
updated_at: 2026-03-27T16:24:18.778287+00:00
parent: SMET-S-0001
blocked_by: [SMET-I-0093]
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: post-initialization-guided-setup
---

# Post-Initialization Guided Setup: Brownfield Analysis, ProductDoc, and Architecture Selection

## Context

When a user runs `initialize_project` on an existing codebase, the current experience is:
1. `.cadre/` directory is created
2. Nothing else happens
3. The agent has no guidance on what to do next
4. The user must manually request brownfield evaluation, ProductDoc creation, architecture selection, etc.

The `project-patterns` skill documents the ideal flow for brownfield and greenfield projects, but nothing **triggers** it. The SessionStart hook checks for active work items but has no special path for "just initialized, no documents yet." The result is that a sophisticated set of tools (`evaluate_brownfield`, `query_architecture_catalog`, `index_code`, `capture_quality_baseline`) sit unused unless the agent or user manually knows to invoke them.

**Depends on:** SMET-I-0093 (BootstrapFlow exposure) — this initiative consumes the bootstrap analysis to drive the guided setup flow.

## Goals & Non-Goals

**Goals:**
- Create a post-init skill/command that automatically guides the user through setting up a new Cadre project
- For brownfield projects: analyze the codebase, present findings, interactively create a ProductDoc, suggest and create a ReferenceArchitecture
- For greenfield projects: interactively create a ProductDoc with the user's vision
- Detect "initialized but empty" state in SessionStart and prompt the guided setup
- Chain the existing tools (`evaluate_brownfield`, `query_architecture_catalog`, `index_code`) into a coherent workflow

**Non-Goals:**
- Modifying the Rust MCP tools themselves (that's SMET-I-0093)
- Fully autonomous document creation without user input — the flow should be interactive
- Replacing existing skills (project-patterns, help) — this builds on top of them

## Use Cases

### Use Case 1: Brownfield Initialization
- **Actor**: Developer initializing Cadre on an existing Rust CLI project
- **Scenario**:
  1. User runs `initialize_project` on their project
  2. Response includes bootstrap analysis (from SMET-I-0093): "Detected Rust CLI tool, cargo build, clippy, rustfmt"
  3. Post-init skill activates automatically
  4. Skill runs `evaluate_brownfield` → finds matching catalog pattern
  5. Skill presents findings: "This looks like a Rust CLI tool. I found these patterns: [list]. Here's what I understand about the product: [summary]."
  6. Skill asks: "Would you like me to create a ProductDoc based on this analysis? What's the product vision?"
  7. User provides input, skill creates and populates ProductDoc interactively
  8. Skill asks: "The architecture catalog has a matching 'rust-cli-tool' pattern. Should I create a ReferenceArchitecture from it?"
  9. User confirms, skill creates ReferenceArchitecture
  10. Skill optionally runs `index_code` and `capture_quality_baseline`
- **Expected Outcome**: Project has a populated ProductDoc, ReferenceArchitecture, and code index within minutes of initialization

### Use Case 2: Greenfield Initialization
- **Actor**: Developer starting a new project from scratch
- **Scenario**:
  1. User runs `initialize_project` on an empty directory
  2. Bootstrap detects greenfield (no source files)
  3. Post-init skill activates
  4. Skill asks: "This is a new project. Let's create a ProductDoc. What problem are you solving?"
  5. Interactive Q&A to build out the ProductDoc
  6. Skill asks about technology choices → suggests architecture catalog patterns
  7. User selects, skill creates ReferenceArchitecture
- **Expected Outcome**: New project has foundational documents before any code is written

### Use Case 3: Returning to Unfinished Setup
- **Actor**: Developer who initialized yesterday but didn't finish setup
- **Scenario**:
  1. SessionStart hook detects `.cadre/` exists but no ProductDoc
  2. Hook injects: "This project was initialized but has no ProductDoc. Run `/cadre-setup` to continue the guided setup."
- **Expected Outcome**: User is reminded to complete setup, not left in limbo

## Detailed Design

### 1. Post-Init Skill: `cadre-setup`

A new skill in `plugins/cadre/skills/` that orchestrates the guided setup flow:

```
Step 1: Read bootstrap analysis (from init response or run analyze_project)
Step 2: Determine brownfield vs greenfield
Step 3: For brownfield:
  a. Run evaluate_brownfield with detected language/project_type
  b. Run index_code to understand the codebase structure
  c. Present findings to user
  d. Interactively create ProductDoc based on code analysis + user input
  e. If catalog match found, offer to create ReferenceArchitecture
  f. Optionally capture initial quality baseline
Step 4: For greenfield:
  a. Interactive ProductDoc creation via Q&A
  b. Technology selection → architecture catalog browsing
  c. Create ReferenceArchitecture from selected pattern
Step 5: Summarize what was created and suggest next steps (create first Epic)
```

### 2. SessionStart Hook Enhancement

Modify `plugins/cadre/hooks/session-start-hook.sh` to detect the "initialized but no ProductDoc" state:

```bash
# After existing checks...
if [ -d ".cadre" ] && [ -z "$(cadre list --type product_doc 2>/dev/null)" ]; then
  echo "## Setup Incomplete"
  echo "Project initialized but no ProductDoc found."
  echo "Run \`/cadre-setup\` to complete guided setup."
fi
```

### 3. Tool Chaining

The skill chains existing MCP tools in sequence:
- `analyze_project` (new from SMET-I-0093) → project understanding
- `evaluate_brownfield` → architecture pattern matching
- `query_architecture_catalog` → find matching patterns
- `index_code` → codebase symbol index
- `create_document` (type: product_doc) → ProductDoc
- `create_document` (type: reference_architecture) → ReferenceArchitecture
- `capture_quality_baseline` → initial quality snapshot

Each step presents results and asks for user confirmation before proceeding.

## Alternatives Considered

1. **Fully automatic setup (no user interaction)** — Rejected because ProductDoc creation requires human intent and vision. The agent can infer structure from code but not product goals.

2. **Embed the flow in initialize_project itself** — Rejected because MCP tools should be atomic. The guided flow is a multi-step interactive process that belongs in the plugin skill layer, not in a single tool call.

3. **Rely on project-patterns skill as-is** — Rejected because it only documents what should happen. Nothing triggers it or chains the tools together. Users must already know the workflow to follow it.

4. **Post-init hook instead of skill** — Rejected because hooks can't do interactive Q&A. A skill allows the agent to pause, ask questions, and adapt based on user responses.

## Implementation Plan

1. Create `cadre-setup` skill with the guided setup flow
2. Enhance SessionStart hook to detect incomplete initialization
3. Implement brownfield setup path (analyze → present → create ProductDoc → suggest architecture)
4. Implement greenfield setup path (Q&A → create ProductDoc → browse catalog → create architecture)
5. Wire `cadre-setup` to be suggested automatically in `initialize_project` response
6. Test end-to-end with a brownfield project (e.g., `crates/cadre-cli`) and a greenfield directory