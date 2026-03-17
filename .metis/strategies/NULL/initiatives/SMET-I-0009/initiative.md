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

# Extend MCP Tools for the Stronger Engineering Model

## Context

Metis currently exposes MCP tools for document CRUD, phase transitions, search, and archival. Super-Metis introduces many new document types, quality operations, rule management, work leasing, and traceability queries. The MCP tool surface must be extended to expose all new capabilities to AI agents.

The MCP server is a primary interface for Super-Metis — AI agents interact with the system primarily through MCP tools. The tool design must be clear, well-documented, and provide rich system prompt instructions so agents know how to use the tools correctly.

## Governing Commitments

This initiative directly serves:
- **Single-agent and orchestrated modes share one governance model.** MCP tools are the primary interface for AI agents. The same tools serve a single focused agent and a coordinated fleet — governance semantics are consistent regardless of execution mode.
- **All durable project memory lives in the repo.** Every MCP tool operation creates or queries repo-native artifacts. Agents interact with the persistent state, not ephemeral context.
- **Structural guidance over improvisation** (Principle #3). The tool surface encodes governance into operations: quality tools expose gate checks and baseline comparisons, rule tools expose protected rules and change proposals, lease tools enforce ownership before execution. System prompt documentation makes these workflows discoverable, but the tools themselves enforce correct sequencing.
- **Governance and quality semantics remain consistent across execution modes** (Vision #9). Quality tools expose the same gate checks, rule tools enforce the same protections, and lease tools require the same ownership — whether the caller is a single focused agent or one runner in an orchestrated fleet.

## Goals & Non-Goals

**Goals:**
- Extend MCP tools to support all new Super-Metis document types
- Add MCP tools for quality operations (capture baseline, compare, check gates)
- Add MCP tools for rule management (query rules, propose changes)
- Add MCP tools for work leasing (acquire, release, check lease status)
- Add MCP tools for traceability queries
- Improve system prompt generation to include complete usage instructions
- Ensure tool responses include enough context for agents to make informed decisions

**Non-Goals:**
- Building a REST API — MCP is the API surface
- Implementing tool-specific UIs — that's SMET-I-0011
- Defining the domain model — that's SMET-I-0001

## Detailed Design

### What to Reuse from `metis/`
- The existing MCP server implementation and tool registration pattern
- Tool response formatting conventions
- System prompt generation infrastructure
- The current tool set as a base (create, read, edit, list, search, transition, archive, reassign)

### What to Change from `metis/`
- Extend `create_document` to handle all new document types
- Update `transition_phase` to handle new phase flows
- Extend `list_documents` and `search_documents` to support new type filters
- Update system prompt to document new document types, phase flows, and workflows

### What is Net New
- Quality tools: `capture_baseline`, `compare_baselines`, `check_quality_gate`, `list_quality_records`, `record_validation`
- Rule tools: `query_rules`, `propose_rule_change`, `approve_rule_change`, `query_rules_by_scope`
- Traceability tools: `trace_ancestry`, `trace_descendants`, `list_cross_references`, `query_execution_records`, `query_transitions`, `query_decisions`
- Investigation tools: `create_investigation`, `link_investigation_to_baseline`
- Design tools: `list_design_contexts`, `link_design_reference`
- Note tools: `fetch_notes_by_scope`, `create_note`, `score_note`, `list_notes`, `propose_note_update`
- Gate/autonomy tools: `check_gate`, `get_autonomy_mode`, `set_autonomy_mode`, `list_escalations`
- Workflow tools: `list_workflow_templates`, `query_operations`, `query_loops`
- Lease tools (post-MVP): `acquire_lease`, `release_lease`, `check_lease_status`, `list_leases`
- Enhanced system prompt with complete Super-Metis workflow documentation including cognitive operations, gates, autonomy modes, and note system

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