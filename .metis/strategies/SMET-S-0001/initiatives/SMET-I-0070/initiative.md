---
id: architecture-mcp-tools
level: initiative
title: "Architecture MCP Tools"
short_code: "SMET-I-0070"
created_at: 2026-03-18T19:17:42.859881+00:00
updated_at: 2026-03-18T19:17:42.859881+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: architecture-mcp-tools
---

# Architecture MCP Tools

## Context

The Architecture document type (SMET-I-0068) and lifecycle hooks (SMET-I-0069) provide the core domain model and automation. This initiative exposes three MCP tools that allow AI agents and external consumers to interact with architecture context: reading it, unlocking documents for editing, and running on-demand conformance checks.

These tools complement the existing `read_document` and `edit_document` tools with architecture-specific operations that understand the document hierarchy and lock semantics.

Design spec: `docs/superpowers/specs/2026-03-18-architecture-in-work-lifecycle-design.md`
Depends on: SMET-I-0068 (Architecture document type), SMET-I-0069 (lifecycle hooks)

## Goals & Non-Goals

**Goals:**
- Implement `get_architecture_context` MCP tool — reads Architecture document for a Story or Task, walking the hierarchy for Tasks
- Implement `unlock_architecture` MCP tool — unlocks an Architecture document for editing with required reason
- Implement `check_conformance` MCP tool — runs conformance check on demand, returns score + violations
- Wire all three tools into the cadre-mcp tool registry
- Integration tests for each tool

**Non-Goals:**
- The Architecture document type (covered by SMET-I-0068)
- The lifecycle hooks (covered by SMET-I-0069)
- CLI commands for architecture operations (future work)
- Plugin skills that consume these tools (plugin layer responsibility)

## Detailed Design

### Tool 1: get_architecture_context

**Purpose**: Read the Architecture document for a given Story or Task. For Tasks, automatically walks up to the parent Story to find the Architecture document.

**Parameters**:
- `project_path: String` (required) — path to .metis directory
- `short_code: String` (required) — Story or Task short code

**Returns**: Full Architecture document content including:
- Source reference architecture
- Relevant layers, boundaries, dependency rules, naming conventions, anti-patterns
- Checklist items with any answers
- Lock status
- Baseline and completion scores
- Or "No architecture context found" message if none exists

**Implementation**:
1. Read the document at short_code
2. If it's a Task, get parent_id to find the Story
3. Query for Architecture documents with parent_id matching the Story
4. If found, return full content; if not, return informational message

### Tool 2: unlock_architecture

**Purpose**: Unlock an Architecture document for editing. Architecture documents are locked by default and require explicit unlock with a reason.

**Parameters**:
- `project_path: String` (required) — path to .metis directory
- `short_code: String` (required) — Architecture document short code (AR-prefixed)
- `reason: String` (required) — why the unlock is needed

**Returns**: Confirmation with unlock record (actor, timestamp, reason). The document remains unlocked until the next `edit_document` call, after which it auto-locks.

**Implementation**:
1. Read the Architecture document
2. Validate it's an Architecture type (AR prefix)
3. Set locked = false
4. Append UnlockRecord to unlock_history
5. Save and return confirmation

### Tool 3: check_conformance

**Purpose**: Run architecture conformance check on demand, independent of phase transitions. Useful for monitoring, investigation, and ad-hoc quality checks.

**Parameters**:
- `project_path: String` (required) — path to .metis directory

**Returns**:
- Conformance score (0.0 - 1.0)
- Total violations count
- Total warnings count
- List of violated rules with details
- Comparison to baseline if a ReferenceArchitecture with baseline exists

**Implementation**:
1. Find the published ReferenceArchitecture
2. If none exists, return informational message
3. Run ArchitectureConformanceChecker
4. Format and return results

### Tool Registration

All tools registered in `crates/cadre-mcp/src/tools.rs` following the existing pattern for `create_document`, `read_document`, etc. Each tool gets:
- Name and description for MCP discovery
- Input schema (JSON Schema for parameters)
- Handler function that calls into cadre-core

## Alternatives Considered

1. **Expose via existing tools only**: Use `read_document` and `edit_document` for all architecture operations. Rejected — hierarchy walking (Task → Story → Architecture) and lock semantics need dedicated tools. `edit_document` doesn't enforce unlock-first semantics.
2. **Single omnibus architecture tool**: One tool with an `action` parameter (get/unlock/check). Rejected — separate tools are more discoverable and have clearer parameter schemas.

## Implementation Plan

Phase 1: get_architecture_context tool with hierarchy walking
Phase 2: unlock_architecture tool with lock/unlock semantics
Phase 3: check_conformance tool with on-demand conformance checking
Phase 4: Wire all tools into MCP tool registry
Phase 5: Integration tests

## Acceptance Criteria

- [ ] get_architecture_context returns correct Architecture document for both Stories and Tasks
- [ ] get_architecture_context returns informational message when no Architecture document exists
- [ ] unlock_architecture correctly unlocks, records reason, and re-locks after edit
- [ ] unlock_architecture rejects non-Architecture documents
- [ ] check_conformance runs conformance check and returns structured results
- [ ] check_conformance handles missing ReferenceArchitecture gracefully
- [ ] All three tools are discoverable via MCP tool listing
- [ ] Integration tests pass for all tools

## Risks / Dependencies

- Depends on SMET-I-0068 for the Architecture document type
- Depends on SMET-I-0069 for hooks that create Architecture documents (tools need documents to operate on)
- MCP tool schema must be compatible with existing tool patterns in cadre-mcp

## Cadre ADR Alignment (SMET-A-0001)

**Audit date**: 2026-03-23 | **Recommendation**: Update dependencies + rename

ADR point 6: This initiative is now a dependency of SMET-I-0078 (Quality Integration, Phase 4). The MCP tools here are the agent-facing interface consumed during `/cadre-execute` review stages.

ADR point 1 (rename): Tool names change — `mcp__cadre__get_architecture_context` → `mcp__cadre__get_architecture_context`, etc. Crate path: `crates/cadre-mcp/` → `crates/cadre-mcp/`. Applied by I-0074.