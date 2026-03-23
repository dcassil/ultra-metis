---
id: build-fresh-subagent-dispatch-in
level: task
title: "Build fresh-subagent dispatch in cadre-ralph-initiative command"
short_code: "SMET-T-0168"
created_at: 2026-03-23T21:11:14.486834+00:00
updated_at: 2026-03-23T21:14:03.854578+00:00
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

# Build fresh-subagent dispatch in cadre-ralph-initiative command

## Parent Initiative
[[SMET-I-0076]]

## Objective
Rewrite the cadre-ralph-initiative command markdown to use the Agent tool for fresh-subagent-per-task dispatch instead of running tasks inline in a single ralph loop session. The command reads the initiative, lists child tasks, then for each task: reads the Cadre doc, maps story type to superpowers skills, and dispatches a fresh Agent with curated context. The SubagentStart hook handles Cadre awareness automatically.

## Scope
- Rewrite `plugins/cadre/commands/cadre-ralph-initiative.md` command instructions
- The command instructs Claude to:
  1. Read the initiative and list child tasks via MCP tools
  2. For each task in `todo` phase:
     - Read the task doc
     - Determine skill mapping from story_type (reuse setup-cadre-ralph.sh logic)
     - Dispatch fresh Agent with: task content + "Invoke skill: superpowers:..." instructions
     - Agent works, updates Cadre doc with progress, commits
     - Transition task to completed on return
  3. Update initiative doc with progress between tasks
  4. Signal completion when all tasks done
- The command uses `allowed-tools` to permit Agent tool usage
- Story-type-to-skill mapping table embedded directly in the command markdown (no need for bash script in agent dispatch path)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] Command dispatches fresh Agent per task (not inline ralph loop)
- [ ] Each subagent gets task doc content + mapped superpowers skill instructions
- [ ] Subagents invoke superpowers skills (TDD, debugging, verification, etc.)
- [ ] Cadre documents updated with progress after each task
- [ ] Tasks transitioned through phases correctly
- [ ] Initiative doc updated between tasks

## Status Updates