---
id: add-optional-review-flag-for-inter
level: task
title: "Add optional --review flag for inter-task code review subagent"
short_code: "SMET-T-0169"
created_at: 2026-03-23T21:11:15.343278+00:00
updated_at: 2026-03-23T21:14:04.108272+00:00
parent: SMET-I-0076
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0076
---

# Add optional --review flag for inter-task code review subagent

## Parent Initiative
[[SMET-I-0076]]

## Objective
Add a `--review` flag to the cadre-ralph-initiative command that dispatches a code review subagent (using `superpowers:requesting-code-review` or the `superpowers:code-reviewer` agent) after each task completes. The reviewer checks the git diff from the task's work and reports issues. If issues found, the orchestrator can re-dispatch a fix agent or escalate to the user.

## Scope
- Add `--review` argument parsing to the command markdown
- After each task's Agent returns, if `--review` is set:
  1. Get the git diff of changes made by that task's agent
  2. Dispatch a code-reviewer Agent with the diff + task acceptance criteria
  3. If reviewer approves: proceed to next task
  4. If reviewer finds issues: report to user (don't auto-retry for MVP)
- The review agent uses `superpowers:requesting-code-review` skill

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] `--review` flag accepted by the command
- [ ] Code review Agent dispatched after each task when flag is set
- [ ] Reviewer gets the git diff and task acceptance criteria
- [ ] Review results logged to the task's Cadre document
- [ ] Without `--review`, behavior is unchanged (no review between tasks)

## Status Updates