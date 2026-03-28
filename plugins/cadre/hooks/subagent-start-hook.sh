#!/bin/bash
# SubagentStart hook for Cadre projects
# Injects concise Cadre context into spawned subagents so they know about
# MCP tools, active work items, and the no-TodoWrite rule.

# Exit silently if not in a Cadre project
if [ ! -d "$CLAUDE_PROJECT_DIR/.cadre" ]; then
    exit 0
fi

# Read hook input from stdin
# shellcheck disable=SC2034
HOOK_INPUT=$(timeout 5 cat 2>/dev/null || true)

# Get active work items if cadre CLI is available
ACTIVE_WORK=""
if command -v cadre &> /dev/null; then
    cd "$CLAUDE_PROJECT_DIR" || exit 0
    STATUS_OUTPUT=$(cadre status --format compact 2>/dev/null)
    if [ -z "$STATUS_OUTPUT" ]; then
        STATUS_OUTPUT=$(cadre status 2>/dev/null | grep -E "^[A-Z]+-[A-Z]+-[0-9]+")
    fi
    ACTIVE_WORK=$(echo "$STATUS_OUTPUT" | grep -E "(active|todo|blocked)" | head -5)
fi

# Build active items section
ACTIVE_SECTION=""
if [ -n "$ACTIVE_WORK" ]; then
    ACTIVE_SECTION="Active items: ${ACTIVE_WORK}"
fi

# Build concise context (~300 tokens)
read -r -d '' CONTEXT << 'EOF'
This is a Cadre project (.cadre directory). Use Cadre MCP tools for all work tracking:
- mcp__cadre__read_document / mcp__cadre__edit_document - Read and update documents
- mcp__cadre__list_documents / mcp__cadre__search_documents - Find documents
- mcp__cadre__transition_phase - Move documents through phases
- mcp__cadre__create_document - Create new documents (MUST populate content after creation)
Do NOT use TodoWrite - use mcp__cadre__edit_document to update active Cadre documents instead.
When working on a task, update its Status Updates section with progress regularly.
EOF

if [ -n "$ACTIVE_SECTION" ]; then
    CONTEXT="${CONTEXT}
${ACTIVE_SECTION}"
fi

# Output JSON for Claude
cat << ENDJSON
{
    "hookSpecificOutput": {
        "hookEventName": "SubagentStart",
        "additionalContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
    }
}
ENDJSON

exit 0
