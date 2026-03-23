---
description: "Decompose an Cadre epic into typed stories"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-cadre-decompose.sh:*)", "mcp__cadre__read_document"]
hide-from-slash-command-tool: "true"
---

# Cadre Decompose - Epic to Stories

## Step 1: Verify Epic Exists

**BEFORE starting**, verify the epic exists.

Parse the SHORT_CODE from: `$ARGUMENTS`

Use `mcp__cadre__read_document` to verify:
- `project_path`: Auto-detect `.metis` directory (usually `$PWD/.metis` or parent)
- `short_code`: The SHORT_CODE from arguments

**If NOT found**: Tell the user the epic was not found. Stop here.
**If found**: Proceed to Step 2.

## Step 2: Initialize Loop

Execute the setup script (reads epic via CLI, builds decomposition prompt, creates ralph-loop state file):

```bash
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-cadre-decompose.sh" $ARGUMENTS
```

## Step 3: Decompose

You are now in a Ralph loop for decomposition. Follow the instructions output by the setup script.

Key points:
- **Invoke each listed skill using the Skill tool** (brainstorming + decomposition)
- **Present the proposed story breakdown to the human BEFORE creating stories**
- Each story MUST have a `story_type` (feature, bugfix, refactor, migration, etc.)
- Each story MUST be immediately populated with real content - no placeholders
- Log progress to the epic document using `mcp__cadre__edit_document`
- When decomposition is FULLY complete, output: `<promise>DECOMPOSITION COMPLETE</promise>`

## Critical Rules

- **ALWAYS** get human approval before creating stories
- **ALWAYS** set story_type for each story
- **ALWAYS** populate content immediately after creation
- **DO NOT** transition the epic to "active" - user will review
- Do NOT create overly granular stories
- Do NOT lie or output false promises to escape the loop
