---
id: end-to-end-test-cadre-setup-on
level: task
title: "End-to-End Test: cadre-setup on Brownfield and Greenfield Projects"
short_code: "SMET-T-0196"
created_at: 2026-03-27T16:06:16.229006+00:00
updated_at: 2026-03-27T16:24:18.248210+00:00
parent: SMET-I-0094
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0094
---

# End-to-End Test: cadre-setup on Brownfield and Greenfield Projects

## Parent Initiative

[[SMET-I-0094]]

## Objective

Validate the complete cadre-setup flow works end-to-end by testing on two scenarios: a brownfield project (the existing `crates/cadre-cli` which was initialized earlier in this session) and a greenfield empty directory. Verify the SessionStart hook correctly detects incomplete setup.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Brownfield test: `initialize_project` on `crates/cadre-cli` returns bootstrap analysis with enriched response
- [ ] Brownfield test: `/cadre-setup` skill triggers and presents correct analysis
- [ ] Greenfield test: `initialize_project` on an empty temp directory returns greenfield classification
- [ ] Greenfield test: `/cadre-setup` skill triggers and starts the ProductDoc Q&A flow
- [ ] SessionStart hook test: shows "Setup Incomplete" when `.cadre/` exists but no ProductDoc
- [ ] SessionStart hook test: does NOT show the message after a ProductDoc is created

## Implementation Notes

### Technical Approach

1. **Brownfield test** — Use the `.cadre/` already initialized in `crates/cadre-cli`. Call `analyze_project` via MCP and verify the response includes Rust, CliTool, Cargo, clippy, etc.

2. **Greenfield test** — Create a temp directory, run `initialize_project`, verify it says "Greenfield" and includes suggested next steps.

3. **SessionStart hook test** — Run the hook script manually and verify output includes "Setup Incomplete" for a project with no ProductDoc. Then create a ProductDoc and verify the message disappears.

4. Fix any issues found during testing.

### Dependencies
- T-0194 (cadre-setup skill) and T-0195 (SessionStart hook) must be complete

## Status Updates

*To be added during implementation*