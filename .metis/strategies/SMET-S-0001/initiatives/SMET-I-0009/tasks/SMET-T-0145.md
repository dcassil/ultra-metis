---
id: extend-anydocument-and-store-to
level: task
title: "Extend AnyDocument and Store to Support Governance Document Types"
short_code: "SMET-T-0145"
created_at: 2026-03-20T17:47:09.390006+00:00
updated_at: 2026-03-20T20:43:08.596604+00:00
parent: SMET-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0009
---

# Extend AnyDocument and Store to Support Governance Document Types

## Context

The `AnyDocument` enum in `crates/cadre-store/src/store.rs` currently only supports Vision, Initiative, and Task. The `DocumentStore::parse_document()` method returns `Err` for all other document types (Epic, Story, Adr, ProductDoc, DesignContext, Specification, and all governance/architecture types). Similarly, `create_document()` only handles Vision, Initiative, and Task creation.

Before any new MCP tools can work, the store must be able to read, write, list, and transition governance document types. This is the foundational task that unblocks all other tasks in this initiative.

## Implementation Plan

1. **Extend `AnyDocument` enum** in `store.rs` to add variants for governance types that need MCP access:
   - `AnalysisBaseline` (quality baselines)
   - `QualityRecord` (quality comparison records)
   - `RulesConfig` (engineering rules)
   - `DurableInsightNote` (insight notes)
   - `CrossReference` (traceability cross-references)
   - `ArchitectureCatalogEntry` (architecture catalog)
   - `ReferenceArchitecture` (selected architecture)

2. **Implement `DocumentCore`-compatible accessors** for each new variant in all `AnyDocument` methods: `short_code()`, `title()`, `document_type()`, `phase()`, `parent_id()`, `archived()`, `to_content()`, `transition_phase()`, `to_summary()`, `full_content()`.

3. **Extend `detect_type_from_content()`** to recognize new `level:` values from frontmatter (e.g., `analysis_baseline`, `quality_record`, `rules_config`, `durable_insight_note`, `cross_reference`, `architecture_catalog_entry`, `reference_architecture`).

4. **Extend `detect_type_from_short_code()`** to handle new type prefixes if they use distinct ones (check existing short code prefix conventions).

5. **Extend `parse_document()`** to construct the correct typed document from file content for each new variant.

6. **Extend `create_document()`** to support creating new instances of governance types with appropriate defaults.

7. **Ensure `list_documents()` and `search_documents()`** correctly include governance documents in results (they iterate all `.md` files, so this mostly means ensuring `parse_document` succeeds).

8. **Add unit tests** for:
   - Round-trip create/read for each new document type through the store
   - `detect_type_from_content` for each new level value
   - `list_documents` including governance docs in results
   - Phase transitions for governance documents

## Files to Modify

- `crates/cadre-store/src/store.rs` — Primary changes: AnyDocument enum, parse_document, create_document, detect_type_from_content
- `crates/cadre-core/src/domain/documents/types.rs` — May need new DocumentType variants or short_code_prefix mappings
- `crates/cadre-core/src/domain/documents/hierarchy.rs` — Hierarchy validation for new types
- `crates/cadre-core/src/lib.rs` — Ensure governance types are exported

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] AnyDocument enum has variants for all 7 governance/architecture types listed above
- [ ] `parse_document()` successfully parses files for each new type
- [ ] `create_document()` can create new instances of each governance type
- [ ] `list_documents()` includes governance documents in results
- [ ] Phase transitions work for governance documents through the store
- [ ] All existing tests continue to pass
- [ ] New unit tests cover round-trip store operations for each governance type

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0009]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

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

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

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

## Status Updates **[REQUIRED]**

### 2026-03-20: Completed
- Extended `DocumentType` enum with 7 governance/architecture variants: AnalysisBaseline, QualityRecord, RulesConfig, DurableInsightNote, CrossReference, ArchitectureCatalogEntry, ReferenceArchitecture
- Added Display, FromStr, short_code_prefix, valid_transitions_from, phase_sequence for all new types
- Extended `AnyDocument` enum with all 7 new variants and all accessor methods
- Updated `parse_document()` to handle all 7 new types via from_content()
- Updated `create_document()` with sensible defaults for all 7 new types
- Updated `detect_type_from_short_code()` with new prefixes (AB, QR, RC, DIN, XR, ACE, RA)
- Updated `detect_type_from_content()` — already works via DocumentType::FromStr
- Updated `transition_phase_with_options()` force path for all new variants
- Added `core_mut()` accessor to: AnalysisBaseline, QualityRecord, RulesConfig, DurableInsightNote, CrossReference, ArchitectureCatalogEntry, ReferenceArchitecture
- Added `metadata()` accessor to DurableInsightNote, CrossReference (others already had it)
- Updated HierarchyValidator to accept all governance types as cross-cutting (no parent required)
- Updated DocumentFactory with wildcard arm for governance types (they use AnyDocument, not Document trait)
- All 58 store tests pass, all core tests pass