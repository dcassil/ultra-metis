#!/bin/bash
# PreToolUse hook that blocks TodoWrite in Cadre projects.
# Agents should use mcp__metis__edit_document to update Cadre documents instead.

# Only block in Cadre projects
if [ ! -d "$CLAUDE_PROJECT_DIR/.metis" ]; then
    exit 0
fi

cat << 'EOF'
{
    "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": "TodoWrite is disabled in this Cadre project. Use Cadre MCP tools for work tracking instead:\n- mcp__metis__edit_document to update active task/initiative Status Updates\n- mcp__metis__create_document to create new work items\n- mcp__metis__transition_phase to advance document phases"
    }
}
EOF

exit 0
