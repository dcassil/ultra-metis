---
description: "Execute all stories under an Cadre epic sequentially"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-cadre-ralph.sh:*)", "mcp__cadre__read_document", "mcp__cadre__list_documents"]
hide-from-slash-command-tool: "true"
---

# Cadre Ralph Epic - Multi-Story Execution

## Step 1: Verify Epic and List Stories

Parse the SHORT_CODE from: `$ARGUMENTS`

1. Use `mcp__cadre__read_document` to verify the epic exists
2. Use `mcp__cadre__list_documents` to find all stories under this epic
3. Filter to stories in `todo` or `discovery` phase (not yet started)

**If no stories found**: Tell the user to run `/cadre-decompose` first. Stop here.

## Step 2: Execute Stories Sequentially

For EACH story that needs execution (in order):

1. Run the ralph setup script for this story:
```bash
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-cadre-ralph.sh" <STORY_SHORT_CODE>
```

2. Execute the story following the ralph loop instructions
3. When the story completes (promise detected), move to the next story
4. Log progress to the epic document between stories

## Step 3: Complete

When ALL stories are complete:
- Update the epic document with completion summary
- Do NOT transition the epic to "completed" - user will review
- Output: `<promise>TASK COMPLETE</promise>`

## Critical Rules

- Execute stories one at a time, sequentially
- Each story follows its own skill mapping based on story_type
- Log epic-level progress between stories
- If any story gets stuck, escalate to user before continuing
- Do NOT skip stories
