# {{ title }}

*Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

<!-- Why this initiative exists: what problem or opportunity it addresses, and why now.
Link to the parent vision or strategy that motivates this work. -->
{% if parent_title is defined and parent_title != "" %}
**Parent**: {{ parent_title }}{% if parent_short_code is defined and parent_short_code != "" %} ({{ parent_short_code }}){% endif %}

{% endif %}
Describe the context and background for this initiative.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Primary objective 1
- Primary objective 2

**Non-Goals:**
- What this initiative will not address

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

<!-- Delete if not a requirements-focused initiative -->

### User Requirements
- **User Characteristics**: Technical background, experience level
- **System Functionality**: What users expect the system to do
- **User Interfaces**: How users will interact with the system

### System Requirements
- **Functional Requirements**:
  - REQ-001: Functional requirement 1
  - REQ-002: Functional requirement 2
- **Non-Functional Requirements**:
  - NFR-001: Performance requirement
  - NFR-002: Security requirement

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

<!-- Delete if not user-facing -->

### Use Case 1: Use Case Name
- **Actor**: Who performs this action
- **Scenario**: Step-by-step interaction
- **Expected Outcome**: What should happen

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

<!-- Delete if not technically complex or if architecture is documented elsewhere -->

### Overview
High-level architectural approach — components and how they interact.

### Component Diagrams
Describe or link to component diagrams.

### Sequence Diagrams
Describe or link to sequence diagrams for key flows.

## Detailed Design **[REQUIRED]**

<!-- Be specific: name files, modules, APIs, and data shapes.
Include what is built, how it fits into existing architecture, and key design decisions. -->

Technical approach and implementation details.

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

<!-- Delete if testing is covered inline in tasks or by a separate initiative -->

### Unit Testing
- **Strategy**: Approach to unit testing
- **Coverage Target**: Expected coverage percentage
- **Tools**: Testing frameworks and tools

### Integration Testing
- **Strategy**: Approach to integration testing
- **Test Environment**: Where integration tests run

### System Testing
- **Strategy**: End-to-end testing approach
- **User Acceptance**: How UAT will be conducted

## Alternatives Considered **[REQUIRED]**

<!-- What other approaches were evaluated and why they were rejected. -->

- **Alternative 1**: Rejected because ...
- **Alternative 2**: Rejected because ...

## Implementation Plan **[REQUIRED]**

<!-- Phases and milestones. Each phase should have a clear deliverable and exit criterion. -->

Phase 1: Phase name — deliverable/exit criterion
Phase 2: Phase name — deliverable/exit criterion
Phase 3: Phase name — deliverable/exit criterion

## Status Updates **[REQUIRED]**

*To be added during implementation*
