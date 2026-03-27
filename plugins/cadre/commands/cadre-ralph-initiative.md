---
description: "Execute all tasks under a Cadre initiative with fresh subagent per task"
argument-hint: "SHORT_CODE [--review]"
allowed-tools: ["Agent", "mcp__cadre__read_document", "mcp__cadre__list_documents", "mcp__cadre__edit_document", "mcp__cadre__transition_phase"]
hide-from-slash-command-tool: "true"
---

# Cadre Ralph Initiative - Multi-Task Execution with Fresh Subagents

## Step 1: Verify Initiative and List Tasks

Parse the SHORT_CODE and flags from: `$ARGUMENTS`

- `SHORT_CODE`: Required. Initiative short code (e.g., PROJ-I-0001)
- `--review`: Optional. Dispatch code review subagent after each task.

1. Use `mcp__cadre__read_document` to verify the initiative exists and is in `active` or `decompose` phase
2. Use `mcp__cadre__list_documents` to find all child tasks under this initiative
3. Filter to tasks in `todo` phase (not yet started)

**If no tasks found**: Tell the user to run `/cadre-decompose` first. Stop here.

**If initiative is not in active phase**: Transition it to active first.

## Step 2: Execute Tasks via Fresh Subagents

For EACH task in `todo` phase (in order):

### 2a. Read the task
Use `mcp__cadre__read_document` to get the full task content.

### 2b. Determine superpowers skills
Map the task's story_type (or default to generic) to required superpowers skills:

| Story Type | Required Skills |
|-----------|----------------|
| feature | superpowers:brainstorming → superpowers:writing-plans → superpowers:test-driven-development → superpowers:verification-before-completion |
| bugfix | superpowers:systematic-debugging → superpowers:verification-before-completion |
| refactor | superpowers:writing-plans → superpowers:verification-before-completion |
| migration | superpowers:writing-plans → superpowers:verification-before-completion |
| architecture-change | superpowers:brainstorming → superpowers:writing-plans → superpowers:verification-before-completion |
| investigation | superpowers:brainstorming |
| remediation | superpowers:systematic-debugging → superpowers:verification-before-completion |
| setup | superpowers:writing-plans → superpowers:verification-before-completion |
| (default/task) | superpowers:test-driven-development → superpowers:verification-before-completion |

### 2c. Transition task to active
Use `mcp__cadre__transition_phase` to move the task from `todo` → `active`.

### 2d. Dispatch fresh Agent
Use the Agent tool to dispatch a fresh subagent with this prompt:

```
Execute Cadre task: {SHORT_CODE}

## Task Content
{full task document content}

## Required Skills (invoke each using the Skill tool)
{mapped skills list, e.g.:
1. Invoke skill: superpowers:test-driven-development
2. Invoke skill: superpowers:verification-before-completion}

## Instructions
1. Read the task requirements above carefully
2. Invoke the Required Skills above using the Skill tool
3. Implement what the task describes
4. Update the task document with progress using mcp__cadre__edit_document
5. Commit your work with a descriptive message
6. Report what you accomplished when done

## Rules
- Follow the Required Skills - invoke each one
- Update the task's Status Updates section with progress
- Do NOT use TodoWrite - update Cadre documents instead
- If blocked, report the blocker clearly
```

### 2e. Handle Agent result
When the Agent returns:
- Log the result summary to the initiative document via `mcp__cadre__edit_document`
- Transition the task to `completed` via `mcp__cadre__transition_phase`

### 2f. Optional review (if --review flag)
If `--review` was specified:
1. Use the Agent tool to dispatch a code-reviewer subagent:
```
Review the code changes for task {SHORT_CODE}.

Use `superpowers:requesting-code-review` to review the most recent commit(s).
Check against these acceptance criteria from the task:
{task acceptance criteria}

Report: approved or issues found.
```
2. If issues found: report to the user and pause for direction
3. If approved: proceed to next task

### 2g. Update initiative
Use `mcp__cadre__edit_document` to log progress to the initiative document's Status Updates section.

## Step 3: Complete

When ALL tasks are complete:
- Update the initiative document with a completion summary
- Do NOT transition the initiative to "completed" - user will review
- Output: `<promise>TASK COMPLETE</promise>`

## Critical Rules

- Dispatch a **fresh Agent per task** — do NOT execute tasks inline
- Each Agent gets the full task content and mapped superpowers skills
- Log initiative-level progress between tasks
- If any task's Agent reports a blocker, escalate to user before continuing
- Do NOT skip tasks
- Do NOT use TodoWrite — all tracking goes through Cadre documents
