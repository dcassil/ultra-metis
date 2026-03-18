---
id: extend-mcp-tools-for-the-stronger
level: initiative
title: "Extend MCP Tools for the Stronger Engineering Model"
short_code: "SMET-I-0009"
created_at: 2026-03-11T19:59:51.468465+00:00
updated_at: 2026-03-11T19:59:51.468465+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: ultra-metis-core-engine-repo
initiative_id: extend-mcp-tools-for-the-stronger
---

# Extend MCP Tools: Expose Completed Domain Types as Static Tools

## Strategy Update (2026-03-18)

**Revised approach**: Rescoped from "expose everything" to "expose what agents actually need." Many domain types are already built in ultra-metis-core but have no MCP tool access. Priority is making completed work usable through static MCP tools, not adding new domain concepts.

**Key decisions:**
- Focus on exposing existing completed domain types (quality, rules, notes, traceability) as read/query MCP tools
- Skip lease tools entirely (POST-MVP, SMET-I-0023 deferred)
- Skip orchestration/workflow tools (execution handled by plugin skills, not MCP tools)
- Use static tools for as much as possible — typed parameters, structured responses
- Plugin skills handle execution workflows; MCP tools handle state queries and mutations
- System prompt improvements are high-priority for agent usability

## Context

Ultra-Metis has built extensive domain types in ultra-metis-core (quality baselines, rules, notes, traceability records, architecture catalog, operations kernel) but most are only accessible through code. The MCP server currently exposes document CRUD, phase transitions, search, archive, reassign, and index_code. Agents need MCP tool access to query and operate on the richer domain model.

The existing tool set (completed in SMET-I-0055) covers core document operations. This initiative adds the next tier: tools that expose the governance, quality, and knowledge layers.

## Goals & Non-Goals

**Goals:**
- Add MCP tools to query quality baselines and records
- Add MCP tools to query engineering rules by scope
- Add MCP tools to fetch/create/score durable insight notes
- Add MCP tools for traceability queries (ancestry, descendants, cross-references)
- Add MCP tools for architecture catalog queries
- Improve system prompt with usage guidance for all tools
- Ensure all tools return structured, actionable responses

**Non-Goals:**
- Lease tools (POST-MVP)
- Orchestration/execution tools (handled by plugin skills)
- Gate/autonomy mode tools (premature — gates not yet wired to real workflows)
- Workflow template tools (operations kernel is structural, not yet operational)
- Rule change proposal tools (can wait until rules are actively enforced)

## Alternatives Considered

1. **Expose everything through existing generic tools**: Rejected because specialized tools with clear names and documentation are easier for agents to use correctly.
2. **GraphQL-style query tool instead of many specialized tools**: Deferred — keep tools simple and specific for now.
3. **Separate MCP servers for different concern areas**: Rejected because a single server is easier to configure and manage.

## Implementation Plan

Phase 1: Extend existing tools for new document types
Phase 2: Implement quality operation tools
Phase 3: Implement rule management tools
Phase 4: Implement lease management tools
Phase 5: Implement traceability query tools
Phase 6: Implement investigation and design tools
Phase 7: Rewrite system prompt with complete Super-Metis documentation
Phase 8: Integration test all tools end-to-end

## Acceptance Criteria

- All new document types can be created, read, edited, and transitioned through MCP
- Quality baselines can be captured and compared through MCP tools
- Rules can be queried and change proposals can be submitted through MCP
- Leases can be acquired and released through MCP tools
- Traceability queries work correctly through MCP
- System prompt provides complete, accurate documentation of all tools and workflows
- Existing Metis MCP tool contracts are preserved for backward compatibility

## Risks / Dependencies

- Depends on all domain model work (SMET-I-0018, I-0019, I-0020, I-0004, I-0006, I-0007)
- Depends on new MVP initiatives: SMET-I-0029 (cognitive operations), SMET-I-0030 (notes), SMET-I-0031 (execution records), SMET-I-0032 (gates/autonomy)
- Tool surface is large — risk of inconsistent naming or behavior across tools
- System prompt size may become unwieldy — need to balance completeness with readability
- Lease/orchestration tools are post-MVP — only implement when SMET-I-0023 and SMET-I-0026 are active

## Codebase Areas to Inspect

- `metis/src/mcp/` — existing MCP server and tool implementations
- `metis/src/mcp/tools/` — individual tool definitions
- `metis/src/mcp/system_prompt.rs` or equivalent — system prompt generation
- `metis/src/mcp/server.rs` — tool registration

## Suggested Tasks for Decomposition

1. Extend create/read/edit/list/search tools for new document types
2. Implement quality baseline MCP tools
3. Implement rule management MCP tools
4. Implement lease management MCP tools
5. Implement traceability query MCP tools
6. Implement investigation and design MCP tools
7. Rewrite system prompt with full Super-Metis documentation
8. Add comprehensive MCP tool integration tests