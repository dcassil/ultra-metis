---
id: gates-escalation-and-autonomy-model
level: initiative
title: "Gates, Escalation, and Autonomy Model"
short_code: "SMET-I-0032"
created_at: 2026-03-16T20:06:14.326527+00:00
updated_at: 2026-03-17T01:20:51.570391+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"
  - "#feature-quality"
  - "#category-quality-governance"


exit_criteria_met: false
estimated_complexity: M
strategy_id: cadre-core-engine-repo
initiative_id: gates-escalation-and-autonomy-model
---

# Gates, Escalation, and Autonomy Model

## Context

CADRE needs explicit control points (gates) where work must meet criteria before proceeding, clear escalation triggers for when the system should involve humans, and configurable autonomy modes that determine how much human oversight is required. This is essential for building trust — the system should be predictable about when it will stop and ask, when it will proceed, and when it will refuse.

The same framework must support tight collaboration (early adoption, risky work), mixed mode (default), and autonomous mode (mature repos with strong rules and validations).

## Governing Commitments

- **The system supports multiple autonomy modes** (Vision #13). This initiative implements those modes.
- **Work is complete when required evidence exists** (Vision #14). Gates enforce evidence requirements.
- **Enforced structure is more important than prompt-only behavior** (Principle #3). Gates make it structurally hard to skip required checks.

## Goals & Non-Goals

**Goals:**
- Define and implement 7 major gates as abstract control points: entry gate, context sufficiency gate, solution gate, execution readiness gate, validation gate, completion gate, escalation gate
- Gates should be attachable to different workflow templates (from SMET-I-0029) — not hardcoded to specific flows
- Define and implement escalation triggers: insufficient evidence, unresolved contradiction, policy conflict, high-impact change, architecture mismatch, security/safety concern, failing required validation, uncertainty above threshold, user/business ambiguity with material impact
- Define and implement 3 autonomy modes:
  - **Tight collaboration**: human approval required at most gates. Best for early adoption, risky repos, architecture change, org-sensitive work.
  - **Mixed mode**: AI proceeds within bounds, escalates on risk or ambiguity. Default mode.
  - **Autonomous mode**: AI proceeds without routine approval, respects gates and thresholds. Only when rules, architecture, validations, and repo maturity are strong.
- Mode should affect: what can be changed directly, what requires approval, how much evidence is required, what contradictions can be tolerated, whether work can be decomposed and dispatched automatically
- Persist autonomy mode configuration as a durable artifact
- Integrate gates with the transition hook system (SMET-I-0007)
- Record all gate checks and escalations in execution records (SMET-I-0031)

**Non-Goals:**
- Implementing the specific quality gates (that's SMET-I-0022) — this initiative provides the gate framework they plug into
- Building the runner that triggers gates — this defines the gate abstraction
- Enforcing autonomy modes at the LLM/prompt level — modes are enforced at the workflow/tool level

## Detailed Design

### Gate Abstraction
Each gate is a typed check point with:
- Gate type (entry, context_sufficiency, solution, execution_readiness, validation, completion, escalation)
- Required evidence (what must exist for the gate to pass)
- Escalation behavior (what happens on failure — block, warn, escalate to human)
- Mode-dependent behavior (how strict the gate is in each autonomy mode)
- Audit trail (gate check result recorded in execution records)

### Escalation Triggers
Each trigger is a condition that, when detected, causes the system to pause and involve a human:
- Insufficient evidence for the current gate
- Unresolved contradiction between artifacts or rules
- Policy conflict (rule says X, context suggests Y)
- High-impact change (affects many files, crosses boundaries)
- Architecture mismatch (proposed change violates reference architecture)
- Security/safety concern
- Failing required validation
- Uncertainty above allowed mode threshold
- User/business ambiguity with material impact

### Autonomy Mode Configuration
```
AutonomyConfig {
  mode: AutonomyMode (tight | mixed | autonomous),
  gate_overrides: Map<GateType, GateBehavior>,  // per-gate customization
  escalation_sensitivity: EscalationLevel,       // how aggressively to escalate
  auto_decompose: bool,                          // can system decompose work without asking
  auto_dispatch: bool,                           // can system dispatch subtasks without asking
  evidence_threshold: EvidenceLevel,             // how much evidence before proceeding
  contradiction_tolerance: ToleranceLevel,       // how much ambiguity is acceptable
}
```

### Mode Behaviors
| Aspect | Tight | Mixed | Autonomous |
|--------|-------|-------|------------|
| Gate approval | Human at most gates | Human at key gates | Auto-pass if criteria met |
| Evidence required | High | Medium | Standard |
| Contradiction tolerance | Very low | Low | Medium |
| Auto-decompose | No | With approval | Yes |
| Auto-dispatch | No | No | Yes |
| Escalation sensitivity | High | Medium | Low |

### Integration Points
- Gates plug into transition hooks (SMET-I-0007) as pre-transition checks
- Gate results recorded in execution records (SMET-I-0031)
- Workflow templates (SMET-I-0029) specify which gates to attach
- Quality gates (SMET-I-0022) are a specific implementation of the validation gate

## Alternatives Considered

1. **Single approval mode**: Rejected — different repos/teams need different levels of oversight. One size doesn't fit all.
2. **Per-task mode configuration**: Deferred — start with per-project mode, add per-task overrides later.
3. **AI self-selects autonomy level**: Rejected — autonomy mode must be a deliberate human choice, not an AI decision.

## Implementation Plan

Phase 1: Define gate abstraction types in Rust
Phase 2: Implement the 7 gate types with mode-dependent behavior
Phase 3: Define escalation trigger types and detection logic
Phase 4: Implement autonomy mode configuration and persistence
Phase 5: Integrate gates with transition hook system (SMET-I-0007)
Phase 6: Integrate gate results with execution records (SMET-I-0031)
Phase 7: Add MCP and CLI tools for mode configuration and gate status
Phase 8: Integration tests for each mode with gate scenarios

## Acceptance Criteria

- All 7 gate types are implemented and attachable to workflows
- All escalation triggers are defined and functional
- Three autonomy modes produce distinct behavior at gates
- Mode configuration is persisted as a durable artifact
- Gate checks are recorded in execution records
- Gates integrate with the transition hook system
- MCP and CLI tools expose mode configuration and gate status queries
- Escalations produce clear, actionable messages for humans

## Risks / Dependencies

- Depends on SMET-I-0007 for transition hook integration
- Depends on SMET-I-0031 for execution record integration
- Depends on SMET-I-0029 for workflow template integration
- Gate configuration complexity — too many knobs becomes unusable. Need sensible defaults.
- Autonomy modes must be clearly explained so users make informed choices
- Must coordinate with SMET-I-0022 (quality gates are a specific gate implementation)

## Suggested Tasks for Decomposition

1. Design gate abstraction trait/struct in Rust
2. Implement 7 gate types with configurable behavior
3. Design escalation trigger types and conditions
4. Implement escalation trigger detection
5. Design autonomy mode configuration schema
6. Implement autonomy mode persistence and loading
7. Implement mode-dependent gate behavior
8. Integrate gates with transition hook system
9. Integrate gate results with execution records
10. Add MCP and CLI tools for mode config and gate queries
11. Integration test all three modes with gate scenarios