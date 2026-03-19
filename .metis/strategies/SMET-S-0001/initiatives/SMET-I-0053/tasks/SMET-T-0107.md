---
id: operations-kernel-cognitive
level: task
title: "Operations Kernel & Cognitive Operations: 12 Operation Types"
short_code: "SMET-T-0107"
created_at: 2026-03-17T22:06:41.511655+00:00
updated_at: 2026-03-17T22:27:56.765386+00:00
parent: SMET-I-0053
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0053
---

# Operations Kernel & Cognitive Operations: 12 Operation Types

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[SMET-I-0053]]

## Objective

Investigate operations kernel and cognitive operations in Ultra-Metis. Document the 12 operation types (frame, acquire, model, locate, analyze, trace, assess, shape, decompose, create, validate, reassess), loop definitions, workflow templates, and execution model.

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

- [x] All 12 cognitive operation types documented with semantics and specifications
- [x] Loop definitions and composition patterns documented
- [x] Workflow template system documented
- [x] Operation equivalents in Metis identified and compared
- [x] Gap analysis completed
- [x] Comparison grids K1-K7 added to SMET-I-0053 section K
- [x] Initiative document updated with comprehensive findings

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

### Investigation Complete

**Date Completed**: 2026-03-17

#### Investigation Summary

Completed comprehensive investigation of the Operations Kernel in Ultra-Metis codebase. All 12 cognitive operation types have been fully documented with specifications, along with loop definitions (11 reusable loops), workflow templates (9 work types with 5 implemented), and execution models.

#### Key Findings

**12 Cognitive Operations** (fully specified):
1. FrameObjective - Establish or refine objectives
2. AcquireContext - Gather and evaluate context
3. BuildModel - Build mental models of systems
4. LocateFocus - Narrow to specific areas
5. AnalyzeStructure - Analyze structure and boundaries
6. TraceFlow - Trace flow and causality
7. AssessImpact - Assess impact and risk
8. ShapeSolution - Shape solution approaches
9. DecomposeWork - Decompose into actionable steps
10. CreateArtifact - Create concrete artifacts (code, config, docs)
11. ValidateReality - Validate against reality (tests, checks, review)
12. ReassessAdapt - Reassess and adapt based on new information

Each operation has complete specifications including:
- Human-readable descriptions
- Input requirements (e.g., "work item", "codebase access")
- Output types (Objective, ContextSet, Model, FocusArea, etc.)
- Tool category hints (Search, Analysis, Testing, Build, VersionControl, Documentation, CodeModification, Metrics)
- Escalation conditions (InsufficientContext, AmbiguityDetected, DesignConflict, etc.)

**11 Reusable Loops** (composition layer):
1. ObjectiveFraming - Establish clear validated objective
2. ContextSufficiency - Gather sufficient context
3. ModelConstruction - Build working model
4. FocusNarrowing - Narrow to specific area
5. Trace - Trace flow and causality
6. RiskImpact - Assess risk and impact
7. SolutionShaping - Shape and refine solution
8. Decomposition - Decompose work into steps
9. ArtifactProduction - Produce concrete artifacts
10. Validation - Validate artifacts against reality
11. Adaptation - Adapt plan based on new information

Each loop specifies:
- Entry/exit conditions (Always, ContextSufficient, ModelSufficient, AllValidationsPass, etc.)
- Default operations composition (2-3 operations per loop)
- Maximum iteration limits (3-10 iterations depending on loop type)
- Escalation rules

**9 Work Types** with **5 Implemented Templates**:
1. Bugfix - Frame → Context → Focus → Trace → RiskImpact → ArtifactProduction → Validation → Adaptation
2. Feature - Frame → Context → Model → SolutionShaping → Decomposition → ArtifactProduction → Validation → Adaptation
3. Refactor - Frame → Model → Focus → RiskImpact → ArtifactProduction → Validation → Adaptation (behavior-preserving)
4. Investigation - Frame → Context → Model → Trace (optional) → ArtifactProduction (findings)
5. Migration - Frame → Model → RiskImpact → Decomposition → ArtifactProduction → Validation → Adaptation

Not yet implemented: ArchitectureChange, BrownfieldEvaluation, Remediation, GreenfieldBootstrap

**Execution Model**:
- Type-safe composition: operations compose into loops, loops compose into workflows
- Condition-based control flow (entry/exit conditions, escalation rules)
- Artifact tracking (required inputs, produced outputs per step)
- Completion rules (AllRequiredLoopsComplete, ArtifactsExist, AllValidationsPass, GateSatisfied, Custom)
- Human escalation: triggers when conditions met (ambiguity, risk, validation failure, iteration budgets)

#### Code Locations

All operations kernel code in: `/Users/danielcassil/projects/ultra-metis/crates/ultra-metis-core/src/domain/operations/`

- `operation.rs` - 12 cognitive operations, OperationSpec, tool categories, escalation conditions
- `loops.rs` - 11 reusable loops, LoopDefinition, entry/exit/escalation rules, iteration limits
- `workflow.rs` - WorkflowTemplate, LoopStep, CompletionRule, 9 work types
- `templates.rs` - 5 built-in workflow templates (bugfix, feature, refactor, investigation, migration)
- `mod.rs` - Module documentation and organization