---
id: architecture-catalog-pattern
level: task
title: "Architecture Catalog & Pattern Selection System"
short_code: "SMET-T-0105"
created_at: 2026-03-17T22:06:40.076240+00:00
updated_at: 2026-03-17T22:34:32.451288+00:00
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

# Architecture Catalog & Pattern Selection System

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Investigate architecture catalog and pattern selection system in Cadre. Document catalog structure, entry format, greenfield/brownfield selection flows, pattern matching algorithms, and reference architecture storage.

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

- [x] Architecture catalog structure and organization documented
- [x] Catalog entry content model documented
- [x] Selection flow (greenfield and brownfield) documented
- [x] Pattern matching and scoring algorithm documented
- [x] Catalog capabilities in Metis identified and compared

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

### Investigation Complete - 2026-03-17

**Execution Status**: All acceptance criteria met. Task transitioned to active and completed with comprehensive findings.

**Comprehensive Comparison Grids Created** (7 grids in SMET-I-0053 section I):

1. **I1. Catalog Structure & Organization**: Compares catalog type, entry format, language support, built-in entries, extensibility, robustness
2. **I2. Greenfield Selection Flow**: Flow model, user interaction, filtering, option presentation, tailoring, persistence
3. **I3. Brownfield Evaluation & Pattern Matching**: Brownfield support, structure analysis, naming detection, matching algorithm, scoring, validation
4. **I4. Reference Architecture Storage**: Storage format, content derivation, tailoring capture, analysis/rules linking
5. **I5. Content Model Comparison**: Title, identifier, language, project type, folder layout, layers, dependency rules, naming conventions, anti-patterns, quality expectations
6. **I6. Cross-System Capability Comparison**: 10 key capabilities with priority ratings (High/Medium/Low)
7. **I7. Implementation Maturity Assessment**: Completeness, rigor, extensibility, automation readiness, enforcement integration, brownfield enablement

**Key Findings**:

- **Metis Strengths**: Broad pattern guidance (greenfield, brownfield, tech-debt, incidents, anti-patterns). Language-agnostic. Human-friendly skill interface.
- **Cadre Strengths**: Typed ArchitectureCatalogEntry structs. Programmatic CatalogQuery + CatalogQueryEngine. Formal StructureAnalyzer → PatternMatcher pipeline with 40/40/20 weighted scoring.
- **Critical Gap**: Original Metis lacks formal brownfield detection system. Cadre implements this for JavaScript only.
- **Future Direction**: Extend Cadre to Rust, Python, Go using same ArchitectureCatalogEntry model. Integrate Metis' broader pattern guidance.

**Code Modules Investigated**:

- `crates/cadre-core/src/domain/catalog/mod.rs` - Catalog module architecture
- `crates/cadre-core/src/domain/catalog/builtin_entries.rs` - 5 JS project types with 30+ validation tests
- `crates/cadre-core/src/domain/catalog/selection_flow.rs` - SelectionFlow (discover → select → create), SelectionOption, TailoringOptions
- `crates/cadre-core/src/domain/catalog/query_engine.rs` - CatalogQuery builder and CatalogQueryEngine with filters
- `crates/cadre-core/src/domain/catalog/brownfield_evaluator/pattern_matcher.rs` - PatternMatcher with PatternMatchScore, MatchResult, 8 validation tests

**Recommendations**:

1. Extend Cadre catalog to Rust, Python, Go (medium effort, high value)
2. Integrate Metis' tech-debt and incident-response patterns into Cadre
3. Create brownfield analysis baselines for common JavaScript patterns
4. Document custom catalog loader for org-specific patterns