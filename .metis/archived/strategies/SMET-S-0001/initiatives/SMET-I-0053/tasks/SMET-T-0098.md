---
id: plugin-architecture-skills-agents
level: task
title: "Plugin Architecture: Skills, Agents, Commands & Hooks"
short_code: "SMET-T-0098"
created_at: 2026-03-17T22:06:36.024423+00:00
updated_at: 2026-03-18T16:58:58.478833+00:00
parent: SMET-I-0053
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0053
---

# Plugin Architecture: Skills, Agents, Commands & Hooks

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Investigate and compare the plugin architectures of original Metis and Ultra-Metis. Map skills, agents, commands, and hooks infrastructure. Document how components are structured, registered, triggered, and integrated. Identify gaps in plugin capabilities between the two systems.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Metis plugin architecture documented: skills, agents, commands, hooks structure and integration
- [x] Ultra-Metis plugin architecture documented: equivalent capabilities and design approach
- [x] Comparison grid created showing feature parity and gaps
- [x] File/module mappings documented for all components
- [x] Gap analysis completed identifying missing capabilities

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates

### Investigation Complete: Plugin Architecture Analysis

**Date Started**: 2026-03-17  
**Completion**: 2026-03-17 (same session)

#### What Was Investigated

Performed comprehensive examination of plugin architectures in both original Metis and Ultra-Metis codebases, focusing on:

1. **Original Metis Plugin System**
   - Reference location: `/Users/danielcassil/projects/ultra-metis/reference - original metis/plugins/metis/`
   - Examined: Plugin manifest, MCP configuration, hook definitions, skill structure, agent design, command structure
   - Key files analyzed: `.claude-plugin/plugin.json`, `.mcp.json`, `hooks/hooks.json`, skills directory structure, agents directory

2. **Ultra-Metis Plugin Architecture**
   - Core location: `crates/ultra-metis-core/src/domain/`
   - MCP location: `crates/ultra-metis-mcp/src/`
   - CLI location: `crates/ultra-metis-cli/src/`
   - Key files analyzed: `transitions/hooks.rs`, `transitions/registry.rs`, `transitions/enforcer.rs`, `protocol.rs`, `tools.rs`, `main.rs`

#### Key Findings

**Plugin Architecture Differences:**
- **Metis**: Distributed markdown + shell script plugin system, declarative JSON-based hooks, skill-based methodology guidance
- **Ultra-Metis**: Rust-native type-safe system with explicit registration, transition-focused hooks, no agents/skills implemented yet

**Specific Component Comparisons:**

1. **Skills System**
   - Metis: 5 skills (document-selection, decomposition, phase-transitions, project-patterns, code-index)
   - Ultra-Metis: Not implemented (gap identified as Medium priority)

2. **Agents System**
   - Metis: flight-levels agent for methodology guidance
   - Ultra-Metis: Placeholder only (gap identified as High priority)

3. **Commands (Ralph Loops)**
   - Metis: /metis-ralph, /metis-decompose, /cancel-metis-ralph for autonomous task execution
   - Ultra-Metis: Not implemented (gap identified as High priority)

4. **Hook System**
   - Metis: General-purpose hooks (SessionStart, PreCompact, PostToolUse, Stop)
   - Ultra-Metis: Transition-specific hooks with type-safe filtering, priority ordering, blocking/warning semantics

5. **MCP Tools**
   - Metis: 8 tools (list, search, read, create, edit, transition, archive, reassign_parent)
   - Ultra-Metis: 8 tools (init, list, search, read, create, edit, transition, archive)

6. **CLI Architecture**
   - Both: Equivalent command structure and coverage
   - Ultra-Metis: Better design (clap framework)

#### Gap Analysis Summary

**Missing in Ultra-Metis (High Priority):**
- Agent system (flight-levels equivalent)
- Ralph loop autonomous execution (state management, stop hook integration)

**Missing in Ultra-Metis (Medium Priority):**
- Skills system (4 methodology guidance skills)
- General-purpose hooks (SessionStart, PreCompact, PostToolUse)

**Where Ultra-Metis Is Stronger:**
- Type-safe hook system with explicit filtering and priorities
- Blocking vs warning semantics in pre-transition checks
- Priority-ordered hook execution (SYSTEM → GATE → USER → ADVISORY)

#### Architecture Philosophy Insights

- **Metis**: Flexible, lightweight, script-based plugin ecosystem designed for rapid iteration
- **Ultra-Metis**: Type-safe, explicit, Rust-native architecture with compile-time guarantees

#### Findings Documented

All findings have been recorded in SMET-I-0053 initiative document under section "#### B. Plugin Architecture & Extensibility" with:
- Detailed comparison grids for each component (B1-B7)
- Original Metis implementations documented with locations
- Ultra-Metis implementations documented with architecture details
- Gap analysis for each area
- Priority and impact assessment

#### Critical Business Insights

1. **Autonomous Execution Gap**: Ultra-Metis lacks Ralph loop capability - critical for AI-in-the-loop workflows
2. **User Guidance Gap**: No agents/skills in Ultra-Metis - reduces discoverability and methodology adoption
3. **Architectural Trade-off**: Ultra-Metis chose type safety over flexibility; Metis chose flexibility for rapid adaptation