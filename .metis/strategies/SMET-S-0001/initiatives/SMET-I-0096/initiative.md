---
id: provider-neutral-agent-surface-adapters
level: initiative
title: "Provider-Neutral Agent Surface: Shared Workflow Core with Thin Claude and OpenAI Adapters"
short_code: "SMET-I-0096"
created_at: 2026-03-27T16:06:17+00:00
updated_at: 2026-03-27T16:06:17+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: provider-neutral-agent-surface-adapters
---

# Provider-Neutral Agent Surface: Shared Workflow Core with Thin Claude and OpenAI Adapters

## Context

Cadre already has a strong portable core:
- `cadre-core` contains the domain model and business logic
- `cadre-store` owns document persistence
- `cadre-mcp` exposes the core as MCP tools
- `cadre` provides a CLI entrypoint

The current coupling is mostly in the agent-facing surface layer. `plugins/cadre/` contains Claude-specific packaging, hooks, slash-command wrappers, Ralph-loop setup scripts, and context injection. Those files are valuable, but they mix three concerns:
- Cadre workflow rules and prompt content that should be shared across providers
- Tool naming and workflow mapping that should be provider-configurable
- Claude-specific host mechanics (`CLAUDE_*` env vars, hook lifecycle, plugin directories, Ralph-loop integration)

This makes OpenAI support look harder than it actually is. The MCP server is already the shared execution surface. The real work is extracting provider-neutral workflow assets from the Claude plugin, then implementing thin adapters for Claude and OpenAI/Codex-style hosts.

This initiative captures that refactor and the first OpenAI adapter so Cadre can support both providers without duplicating workflow logic or reintroducing namespace drift (`mcp__metis__...` vs `mcp__cadre__...`) across prompt assets.

## Goals & Non-Goals

**Goals:**
- Define a provider-neutral Cadre workflow core for agent-facing behavior
- Extract reusable logic from the Claude plugin: context text, story-type-to-skill mapping, workflow recipes, and tool usage guidance
- Keep `cadre-mcp` as the shared tool surface for both Claude and OpenAI adapters
- Preserve the current Claude experience with a thin adapter over the shared workflow core
- Add an OpenAI adapter for Codex/OpenAI-style coding agents that can consume the same MCP tools and workflow guidance
- Eliminate copied prompt drift and stale references across agent surfaces
- Make future provider additions incremental rather than a second full plugin implementation

**Non-Goals:**
- Supporting plain consumer ChatGPT without tool or MCP integration
- Rewriting Cadre's domain model, storage layer, or MCP tool set
- Achieving perfect Claude feature parity in OpenAI on day one if the host lacks equivalent hook/loop primitives
- Shipping a provider marketplace/distribution pipeline for every host
- Designing a generic plugin runtime for arbitrary third-party providers beyond Claude and OpenAI

## Use Cases

### Use Case 1: Claude Remains Fully Supported
- **Actor**: Existing Claude Code user in a Cadre repo
- **Scenario**:
  1. Claude starts in a Cadre project
  2. Claude adapter loads provider-rendered Cadre session context
  3. Claude-specific hooks and commands continue to work
  4. Workflow rules, tool guidance, and story-type mapping come from the shared core rather than handwritten duplicates
- **Expected Outcome**: No behavioral regression for Claude users, with less maintenance and fewer stale references

### Use Case 2: OpenAI/Codex Uses the Same Cadre Workflow
- **Actor**: OpenAI/Codex user working in the same repository
- **Scenario**:
  1. OpenAI adapter provides Cadre project context and workflow instructions in the host's native format
  2. The agent uses the same `cadre-mcp` tools as Claude
  3. The agent follows the same work-tracking rules, document lifecycle guidance, and skill mapping
- **Expected Outcome**: OpenAI users can operate inside Cadre with the same domain model and nearly the same workflow semantics

### Use Case 3: Workflow Rule Changes Apply Once
- **Actor**: Cadre maintainer updating agent guidance
- **Scenario**:
  1. Maintainer changes one shared workflow asset, such as the no-TodoWrite rule or story-type mapping
  2. Claude and OpenAI adapters consume the updated shared definition
  3. Provider-specific output is regenerated or re-rendered from the same source
- **Expected Outcome**: One change updates both providers without copy/paste edits

## Architecture

### Overview

Split the agent-facing system into three layers:

1. **Shared execution core**
   - `crates/cadre-core`
   - `crates/cadre-store`
   - `crates/cadre-mcp`
   - `crates/cadre-cli`

2. **Shared workflow core**
   - Provider-neutral assets describing Cadre workflow behavior:
     - session context
     - subagent context
     - workflow recipes (`decompose`, `execute task`, `guided setup`)
     - story-type-to-skill mappings
     - tool aliases and provider-visible names
     - common warnings and guardrails

3. **Provider adapters**
   - **Claude adapter**: hooks, slash-command wrappers, plugin manifest, Ralph-loop integration
   - **OpenAI adapter**: OpenAI/Codex-native instruction files, helper commands/wrappers, and MCP registration guidance

### Proposed Repo Shape

One concrete option:

```text
crates/
  cadre-core/
  cadre-store/
  cadre-mcp/
  cadre-cli/

surfaces/
  shared/
    context/
    workflows/
    mappings/
    templates/
    renderers/
  claude/
    plugin/
    hooks/
    commands/
    scripts/
  openai/
    instructions/
    commands/
    scripts/
    skills/
```

`plugins/cadre/` can either become the Claude adapter directly or be replaced by `surfaces/claude/` with a compatibility path during migration. The important point is that provider-specific files stop being the source of truth for workflow logic.

### Shared Workflow Core Responsibilities

The shared workflow core should own:
- Canonical Cadre workflow language
- Canonical tool references as logical operations, not host-specific strings
- Canonical story-type-to-methodology mapping
- Canonical session/subagent context content
- Canonical task and initiative execution recipes

The shared workflow core should not own:
- Claude hook JSON format
- Claude-only env vars or plugin root assumptions
- OpenAI host-specific instruction schema
- Loop implementation details specific to a host

### Claude Adapter Responsibilities

The Claude adapter should remain intentionally thin:
- Map shared session context into `SessionStart` and `PreCompact` hook output
- Map shared subagent context into `SubagentStart`
- Render Claude command markdown for `cadre-ralph`, `cadre-decompose`, and related flows
- Keep Ralph-loop integration as a Claude-only execution primitive where useful
- Translate logical tool aliases to Claude-visible MCP names such as `mcp__cadre__read_document`

### OpenAI Adapter Responsibilities

The OpenAI adapter should:
- Inject the same Cadre workflow guidance into OpenAI/Codex-native instruction surfaces
- Use the same `cadre-mcp` server and tool set
- Provide thin helper commands or prompt wrappers for common flows like task execution and decomposition
- Reuse the same tool aliases and mappings rendered into the OpenAI host's naming conventions
- Gracefully degrade where Claude-only loop or hook primitives do not exist

### Key Design Constraint

Do not model OpenAI support as "extending the Claude plugin." Claude and OpenAI should both extend the shared workflow core. Otherwise Claude assumptions will leak into the OpenAI surface and recreate the same portability problem.

## Detailed Design

### 1. Introduce Logical Tool Aliases

Define logical Cadre operations such as:
- `read_document`
- `edit_document`
- `transition_phase`
- `create_document`
- `list_documents`

Provider renderers map those operations to host-visible identifiers:
- Claude: `mcp__cadre__read_document`
- OpenAI: whatever naming the host exposes for the same MCP tool

This removes hardcoded tool strings from prompt assets and prevents rename drift.

### 2. Extract Shared Context Assets

Move the actual instructional content out of provider hooks and command files into shared templates:
- Cadre project detection summary
- no-TodoWrite / use-documents-for-memory rule
- active work summary framing
- document lifecycle guidance
- document creation rules

Claude hook scripts and OpenAI instructions should render these shared assets, not own them.

### 3. Extract Shared Workflow Recipes

Model workflows as reusable definitions:
- task execution
- initiative decomposition
- guided setup
- multi-task orchestration

Each workflow should expose:
- purpose
- required logical tools
- required methodology/skills
- human approval gates
- completion semantics

Provider adapters then wrap those workflows in the host's native delivery mechanism.

### 4. Normalize Story-Type Methodology Mapping

The mapping from `story_type` to methodology is currently embedded in Claude setup scripts. Move it to a shared data file or small renderer input so both adapters use the same mapping.

This should include at minimum:
- `feature` -> brainstorming + planning + implementation verification
- `bugfix` -> systematic debugging + verification
- `refactor` -> planning + verification
- `migration` -> planning + verification
- `architecture-change` -> brainstorming + planning + verification
- `investigation` -> exploration/investigation guidance
- `remediation` -> debugging + verification
- default task behavior

### 5. Leave MCP and CLI as Shared Runtime, Not Provider Assets

No provider should implement duplicate business logic for:
- document reads/writes
- phase transitions
- project indexing
- rules queries
- architecture evaluation

Those continue to live in Rust and remain the single execution backend.

### 6. OpenAI First Adapter Scope

For the first OpenAI adapter release, target a pragmatic parity level:
- Cadre-aware project/session instructions
- MCP tool usage
- task execution guidance
- initiative decomposition guidance
- shared methodology mapping

Explicitly defer Claude-only ergonomics unless the OpenAI host exposes equivalent primitives:
- Ralph-loop behavior
- Claude hook lifecycle
- Claude plugin installation conventions

## Alternatives Considered

1. **Keep Claude as the source of truth and manually port to OpenAI**
   - Rejected because this preserves copy/paste drift, stale tool references, and provider-specific coupling.

2. **Move all agent workflow behavior into Rust**
   - Rejected because host-specific delivery still matters. Rust should own business logic, not every provider UX primitive.

3. **Build OpenAI support directly without extracting a shared core**
   - Rejected because it would be the fastest short-term path but the worst maintenance shape. Two thin adapters over one shared workflow core is the more durable design.

4. **Aim for exact Claude feature parity before shipping OpenAI support**
   - Rejected because it overfits to Claude host capabilities and delays useful OpenAI support. Shared guidance plus MCP access delivers most of the value early.

## Implementation Plan

1. Inventory all provider-specific workflow assets in `plugins/cadre/` and classify each as shared vs Claude-only
2. Create a shared workflow core for context text, logical tool aliases, workflow recipes, and story-type methodology mappings
3. Refactor the Claude adapter to render from the shared workflow core without changing user-visible behavior
4. Fix existing namespace drift in provider assets while doing the extraction (`mcp__metis__...` remnants, stale naming, provider-specific hardcoding)
5. Create the first OpenAI adapter with shared session instructions, task execution guidance, and decomposition guidance
6. Validate both adapters against the same representative workflows: session start, task execution, initiative decomposition, and document updates
7. Document capability differences explicitly where host features diverge

## Testing Strategy

### Shared Layer Validation
- Verify shared mappings render identical logical behavior across both providers
- Snapshot rendered outputs for key workflows to detect accidental drift

### Claude Regression Testing
- Existing Claude session start, subagent start, and command flows still work
- Existing Claude users still see `mcp__cadre__*` guidance and Cadre workflow rules

### OpenAI Adapter Testing
- OpenAI/Codex session can discover and use the Cadre MCP tools
- Agent receives Cadre context and follows document-centric workflow rules
- Task execution and decomposition flows can be performed without Claude-only primitives

## Risks / Dependencies

- Depends on the current `cadre-mcp` tool surface remaining the shared contract
- Must coordinate with existing Claude plugin initiatives so this refactor does not regress working flows
- OpenAI host capabilities may not match Claude hooks or loop mechanics exactly; the adapter must degrade intentionally rather than emulate them poorly
- If shared assets are not made authoritative, drift will reappear quickly

## Status Updates

### 2026-03-27
- Initiative created from architecture review of the current Claude plugin and MCP server
- Initial conclusion: feasibility is high, but the portability work is in the agent-surface layer rather than the MCP server
- Recommended architecture recorded here: shared workflow core plus thin Claude and OpenAI adapters
