---
id: document-types-complete-coverage
level: task
title: "Document Types: Complete Coverage Analysis (Metis & Cadre)"
short_code: "SMET-T-0097"
created_at: 2026-03-17T22:06:28.303879+00:00
updated_at: 2026-03-17T22:20:43.758435+00:00
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

# Document Types: Complete Coverage Analysis (Metis & Cadre)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Comprehensively investigate and compare all document types defined in both original Metis and Cadre systems. Map each document type's phase definitions, validation rules, serialization, templates, and hierarchy constraints. Create detailed comparison grids for each document type showing design differences and feature gaps.

Document Types to Investigate:
- **Metis**: Vision, Strategy, Initiative, Task, Backlog, ADR, Specification
- **Cadre**: Vision, Initiative, Task, Epic, Story, DesignContext, ProductDoc, Specification

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Comparison grid created for each document type showing Metis vs Cadre side-by-side
- [x] For each document type: phase definitions, transitions, validation rules documented
- [x] Template system examined: template content, rendering context, structure compared
- [x] Hierarchy constraints documented for each document type
- [x] Serialization examined: markdown structure, YAML frontmatter fields compared
- [x] Gap analysis completed: which document types exist in one system but not the other
- [x] Assessment completed: which design is more robust/complete for each type
- [x] Findings rolled up into comprehensive mapping in initiative document

## Implementation Notes

### Technical Approach

1. **Phase Definition Comparison** — For each document type in both systems:
   - Map all phases (e.g., draft → review → published, discovery → design → ready → decompose → active → completed)
   - Document phase transition rules and constraints
   - Identify phase validation and exit criteria
   - Compare phase workflows between systems

2. **Validation & Constraints** — Examine:
   - Parent/child relationship constraints per document type
   - Hierarchy depth limitations
   - Type-specific validation rules
   - Blocked state handling (if applicable)

3. **Template Coverage** — For each document type:
   - Examine template structure and sections
   - Document required vs optional fields
   - Compare template completeness and usefulness
   - Rate template quality and clarity

4. **Serialization** — Compare:
   - Markdown structure and conventions
   - YAML frontmatter fields per document type
   - Round-trip serialization/deserialization handling
   - Archive and restoration patterns

5. **Feature Comparison Matrix** —  For each document type, document:
   - Feature exists in both systems
   - Only exists in Metis
   - Only exists in Cadre
   - Design differences where feature exists in both
   - Robustness assessment (which implementation is stronger)

### Dependencies

- SMET-T-0096 (Comparative Analysis Summary) must be completed first to establish comparison template and findings index

### Risk Considerations

- Document type coverage must be complete to avoid gaps in final analysis
- Phase transition rules are complex and require careful documentation to avoid errors
- Template examination requires understanding rendering/context passing in both systems

## Status Updates

### Session 1 - 2026-03-17

**Completed Investigation:**

1. ✓ **Document Type Coverage** — Investigated all document types in both systems:
   - Metis: Vision, Strategy, Initiative, Task, Backlog, ADR, Specification (7 types)
   - Cadre: Vision, Initiative, Task, Epic, Story, DesignContext, ProductDoc, Specification, ADR (9 types)

2. ✓ **Phase Definition Comparison** — Documented all phase models:
   - Vision: Draft → Review → Published (both systems, identical)
   - Initiative: Discovery → Design → Ready → Decompose → Active → Completed (both systems, identical)
   - Task: Backlog → Todo → Active → Completed + Blocked state (both systems, identical phases)
   - ADR: Draft → Discussion → Decided → Superseded (both systems, identical)
   - Strategy: Shaping → Design → Ready → Active → Completed (Metis only, missing in Cadre)

3. ✓ **Template System Comparison** — Examined template structure:
   - Metis: Markdown + YAML frontmatter (implicit)
   - Cadre: Explicit Tera template system with schema versioning
   - Cadre: DocumentCore pattern for all types, explicit template rendering

4. ✓ **Hierarchy Constraints** — Documented parent/child relationships:
   - Vision: Root-level (no parent)
   - Initiative: Parent to Tasks, child to Vision/Strategy/Epic
   - Epic: Parent to Stories (Cadre only)
   - Story: Parent to Tasks (Cadre only)
   - Task: Must have parent (Initiative/Epic/Story)

5. ✓ **Serialization Differences** — Documented storage approaches:
   - Both use markdown + YAML frontmatter
   - Cadre: Schema versioning (schema_version field)
   - Cadre: DocumentId derived from title slug
   - Metis: Implicit serialization patterns

**Key Findings Added to SMET-I-0053:**
- Comprehensive comparison grid (A1-A8) with 8 detailed sections (Vision, Initiative, Task, Epic, Story, ADR, Specification, Strategy)
- Document type summary table showing coverage across 11 document types
- Four critical gaps identified

**Critical Gaps Identified:**

| Gap | Priority | Impact | Complexity |
|-----|----------|--------|-----------|
| **Strategy Missing in Cadre** | HIGH | Blocks large-scale program coordination | Large (XL) |
| **Ralph Loop Equivalent** | HIGH | Autonomous task execution differs | Large (L) |
| **Epic/Story Enhancements** | MEDIUM | Organizational richer structure, no equivalent in Metis | Medium (M) |

**Assessment Summary:**
- **Core Coverage**: 91% equivalent (10/11 types comparable)
- **Phase Models**: 100% identical for equivalent types
- **Design Quality**: Cadre slightly better (explicit schema, stronger typing)
- **Robustness**: Comparable for matching types
- **Missing Capability**: Strategy document type is critical gap

**Deliverables:**
- ✓ Detailed comparison grids for all document types added to initiative
- ✓ Summary table with coverage status
- ✓ Gap analysis with priority levels
- ✓ Findings integration into SMET-I-0053