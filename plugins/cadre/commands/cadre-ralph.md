---
description: "Execute an Cadre story or task with Ralph loop"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-cadre-ralph.sh:*)", "mcp__cadre__read_document"]
hide-from-slash-command-tool: "true"
---

# Cadre Ralph - Story/Task Execution

## Step 1: Verify Document Exists

**BEFORE starting the Ralph loop**, verify the document exists.

Parse the SHORT_CODE from: `$ARGUMENTS`

Use `mcp__cadre__read_document` to verify:
- `project_path`: Auto-detect `.metis` directory (usually `$PWD/.metis` or parent)
- `short_code`: The SHORT_CODE from arguments

**If NOT found**: Tell the user the document was not found. Stop here.
**If found**: Note the document type and story_type, then proceed to Step 2.

## Step 2: Initialize Loop

Execute the setup script (reads document via CLI, maps story type to skills, creates ralph-loop state file):

```bash
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-cadre-ralph.sh" $ARGUMENTS
```

## Step 3: Execute

You are now in a Ralph loop powered by the ralph-loop plugin. Follow the instructions output by the setup script.

Key points:
- The setup script has already mapped the story type to specific superpowers skills
- **Invoke each listed skill using the Skill tool** before starting implementation
- Use `mcp__cadre__edit_document` to log progress after each significant step
- The ralph-loop stop hook will feed the same prompt back on each iteration
- When FULLY complete, output: `<promise>TASK COMPLETE</promise>`

## Critical Rules

- **ONLY** output the promise when work is genuinely complete
- **DO NOT** transition to "completed" - the user will review first
- **ALWAYS** log progress to the document - this is your persistent memory
- **ALWAYS** invoke the mapped superpowers skills - they are not optional
- Do NOT lie or output false promises to escape the loop
