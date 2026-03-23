---
id: extend-mcp-tools-for-the-stronger
level: initiative
title: "Extend MCP Tools for the Stronger Engineering Model"
short_code: "SMET-I-0009"
created_at: 2026-03-11T19:59:51.468465+00:00
updated_at: 2026-03-23T20:41:44.751600+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
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

## Acceptance Criteria

## Acceptance Criteria

- All new document types can be created, read, edited, and transitioned through MCP
- Quality baselines can be captured and compared through MCP tools
- Rules can be queried and change proposals can be submitted through MCP
- Leases can be acquired and released through MCP tools
- Traceability queries work correctly through MCP
- System prompt provides complete, accurate documentation of all tools and workflows
- All tools return structured, actionable responses with consistent naming

## Risks / Dependencies

- Depends on all domain model work (SMET-I-0018, I-0019, I-0020, I-0004, I-0006, I-0007)
- Depends on new MVP initiatives: SMET-I-0029 (cognitive operations), SMET-I-0030 (notes), SMET-I-0031 (execution records), SMET-I-0032 (gates/autonomy)
- Tool surface is large — risk of inconsistent naming or behavior across tools
- System prompt size may become unwieldy — need to balance completeness with readability
- Lease/orchestration tools are post-MVP — only implement when SMET-I-0023 and SMET-I-0026 are active

## Codebase Areas to Inspect

- `crates/ultra-metis-mcp/src/` — MCP server, protocol, and tool implementations
- `crates/ultra-metis-mcp/src/tools.rs` — tool definitions and dispatch
- `crates/ultra-metis-mcp/src/protocol.rs` — JSON-RPC message routing
- `crates/ultra-metis-core/src/domain/` — domain types to expose as tools
- `crates/ultra-metis-store/src/store.rs` — persistence layer the tools call

## Task Breakdown (Decomposed 2026-03-20)

### Foundation Layer
1. **SMET-T-0145**: Extend AnyDocument and Store to Support Governance Document Types
   - Extends `AnyDocument` enum and `DocumentStore` to handle 7 governance/architecture types
   - Unblocks all other tasks

### Domain-Specific MCP Tools (can be parallelized after T-0145)
2. **SMET-T-0146**: Implement Quality Baseline and Record MCP Tools
   - Tools: `capture_quality_baseline`, `compare_quality_baselines`, `list_quality_records`, `check_architecture_conformance`
3. **SMET-T-0147**: Implement Rule Query MCP Tools with Scope Inheritance
   - Tools: `query_rules`, `get_applicable_rules`, `list_protected_rules`
4. **SMET-T-0148**: Implement Durable Insight Note MCP Tools with Scope-Based Fetch and Feedback
   - Tools: `create_insight_note`, `fetch_insight_notes`, `score_insight_note`, `list_insight_notes`
5. **SMET-T-0149**: Implement Traceability and Cross-Reference MCP Tools
   - Tools: `create_cross_reference`, `query_relationships`, `trace_ancestry`, `list_cross_references`
6. **SMET-T-0150**: Implement Architecture Catalog Query MCP Tools
   - Tools: `query_architecture_catalog`, `list_catalog_languages`, `read_reference_architecture`, `evaluate_brownfield`

### Documentation and Testing (after tool implementation)
7. **SMET-T-0151**: Rewrite MCP System Prompt with Complete Tool and Workflow Documentation
   - System prompt via MCP prompts protocol, workflow recipes, domain concept guide
8. **SMET-T-0152**: Integration Tests for All New MCP Tools
   - 30+ integration tests through `call_tool()` dispatch, covering success/error/edge cases

### Dependency Graph
```
T-0145 (foundation) -> T-0146, T-0147, T-0148, T-0149, T-0150 (parallel)
T-0146..T-0150 -> T-0151 (system prompt), T-0152 (integration tests)
```

### Total New MCP Tools: ~18
Existing tools: 10 (initialize, create, read, list, edit, transition, search, archive, index_code, reassign)
New tools: 18 (4 quality + 3 rules + 4 notes + 4 traceability + 4 architecture - 1 overlap)

### Removed from Scope (per Strategy Update)
- Lease management tools (POST-MVP, deferred to SMET-I-0023)
- Orchestration/execution tools (handled by plugin skills)
- Gate/autonomy mode tools (premature)
- Rule change proposal tools (can wait)

## Cadre ADR Alignment (SMET-A-0001)

**Audit date**: 2026-03-23 | **Recommendation**: Update scope (rename)

All 8 tasks under this initiative are completed. The rename from ultra-metis to Cadre (SMET-I-0074) will change:
- MCP tool prefix: `mcp__ultra-metis__` → `mcp__cadre__`
- Crate paths: `crates/ultra-metis-mcp/` → `crates/cadre-mcp/`
- Binary name: `ultra-metis-mcp` → `cadre-mcp`

These are mechanical name changes applied by the rename initiative. No scope or functionality changes needed here.