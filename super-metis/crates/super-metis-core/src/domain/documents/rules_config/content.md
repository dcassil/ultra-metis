# {{ title }}

## Rules

<!-- List each engineering rule. Rules should be clear, testable, and enforceable. -->

### Rule 1: (rule name)
- **Statement**: (what MUST or MUST NOT be done)
- **Rationale**: (why this rule exists)
- **Detection**: (how violations are detected -- tool name and rule ID)
- **Protection**: (standard / protected -- protected rules require DesignChangeProposal to modify)

## Scope

<!-- Define exactly what these rules apply to. -->
- **Platforms**: (e.g., "all Rust crates in this workspace")
- **Components**: (specific packages, modules, or paths)
- **Exclusions**: (what is explicitly excluded from these rules)

## Protection Policy

<!-- How these rules are governed. -->
- **Default Protection**: (standard / protected)
- **Change Process**: (for standard: RuleChangeProposal. For protected: DesignChangeProposal)
- **Escalation**: (who to contact when a rule needs emergency modification)

## Change History

<!-- Dated log of rule changes. -->
| Date | Change | Reason | Proposal |
|------|--------|--------|----------|
| (YYYY-MM-DD) | (what changed) | (why) | (short code of change proposal) |