# {{ title }}

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

<!-- Why this initiative exists: what problem or opportunity it addresses, and why now.
Link to the parent vision or strategy that motivates this work. -->
{% if parent_title is defined and parent_title != "" %}
**Parent**: {{ parent_title }}{% if parent_short_code is defined and parent_short_code != "" %} ({{ parent_short_code }}){% endif %}

{% endif %}
{Describe the context and background for this initiative — the problem, the opportunity, and why it's a priority now}

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- {Primary objective 1 — what this initiative will achieve}
- {Primary objective 2}

**Non-Goals:**
- {What this initiative will explicitly not address — be specific to prevent scope creep}

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete this section if this is not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, role}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system must do}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system must behave}
  - NFR-001: {Performance requirement — e.g., "P99 latency < 200ms under 1k RPS"}
  - NFR-002: {Security or reliability requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

{Delete this section if not user-facing}

### Use Case 1: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

### Use Case 2: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete this section if not technically complex or if architecture is documented elsewhere}

### Overview
{High-level architectural approach — what components are involved and how they interact}

### Component Diagrams
{Describe or link to component diagrams}

### Sequence Diagrams
{Describe or link to sequence diagrams for key interaction flows}

## Detailed Design **[REQUIRED]**

<!-- The technical or process design for this initiative. Be specific: name files, modules, APIs, and data shapes.
For engineering initiatives, include: what is built, how it fits into existing architecture, and key design decisions. -->

{Technical approach and implementation details — specific enough that an engineer can start without additional clarification}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete this section if testing is covered inline in tasks or by a separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}

## Alternatives Considered **[REQUIRED]**

<!-- What other approaches were evaluated and why they were rejected.
Be honest — this prevents re-litigating decisions later. -->

- **{Alternative 1}**: Rejected because {specific reason}
- **{Alternative 2}**: Rejected because {specific reason}

## Implementation Plan **[REQUIRED]**

<!-- Phases and milestones for executing this initiative.
Each phase should have a clear deliverable and exit criterion. -->

Phase 1: {Phase name} — {deliverable/exit criterion}
Phase 2: {Phase name} — {deliverable/exit criterion}
Phase 3: {Phase name} — {deliverable/exit criterion}

## Status Updates **[REQUIRED]**

*To be added during implementation*
