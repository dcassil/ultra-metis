---
id: improve-content-templates-for-all
level: task
title: "Improve Content Templates for All Document Types"
short_code: "SMET-T-0062"
created_at: 2026-03-17T02:01:20.076017+00:00
updated_at: 2026-03-17T02:10:28.945321+00:00
parent: SMET-I-0014
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0014
---

# Improve Content Templates for All Document Types

## Objective

Improve content.md templates for governance, architecture, execution, and quality document types. Replace generic placeholder guidance with specific, actionable section guidance. Each template should be purpose-specific with clear instructions per section that help both humans and AI agents produce high-quality documents.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All governance templates improved (validation_record, remediation_record, constraint_record, approval_record, validation_policy, ownership_map)
- [ ] All architecture templates improved (architecture_investigation, architecture_catalog_entry, reference_architecture)
- [ ] All execution templates improved (execution_record, transition_record, decision_record, durable_insight_note, cross_reference)
- [ ] All quality templates improved (quality_gate_config, gate_override, quality_record, analysis_baseline)
- [ ] No generic conditional sections remain
- [ ] cargo test passes with all updated templates

## Implementation Notes

### Technical Approach
Update each content.md template file with richer, more specific guidance per section. Keep Tera `{{ title }}` variable. Replace `{generic placeholder}` with detailed guidance comments.

## Status Updates

*To be added during implementation*