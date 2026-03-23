#!/bin/bash
# PreCompact hook for Cadre projects
# Re-injects Cadre context after context compaction with current project state

# Exit silently if not in an Cadre project
if [ ! -d "$CLAUDE_PROJECT_DIR/.metis" ]; then
    exit 0
fi

# Check if cadre is installed
if ! command -v cadre &> /dev/null; then
    cat << 'ENDJSON'
{
    "systemContext": "WARNING: This is an Cadre project (`.metis` directory found) but the `cadre` command is not installed or not in PATH. Run `make install` from the cadre repo root."
}
ENDJSON
    exit 0
fi

# Get current project state
cd "$CLAUDE_PROJECT_DIR" || exit 0
STATUS_OUTPUT=$(cadre status --format compact 2>/dev/null)
if [ -z "$STATUS_OUTPUT" ]; then
    STATUS_OUTPUT=$(cadre status 2>/dev/null | grep -E "^[A-Z]+-[A-Z]+-[0-9]+")
fi
ACTIVE_WORK=$(echo "$STATUS_OUTPUT" | grep -E "(active|todo|blocked)" | head -10)
BLOCKED_COUNT=$(echo "$STATUS_OUTPUT" | grep -c "blocked" 2>/dev/null || true)
ACTIVE_COUNT=$(echo "$STATUS_OUTPUT" | grep -c "active" 2>/dev/null || true)
TODO_COUNT=$(echo "$STATUS_OUTPUT" | grep -c "todo" 2>/dev/null || true)
[ -z "$BLOCKED_COUNT" ] && BLOCKED_COUNT=0
[ -z "$ACTIVE_COUNT" ] && ACTIVE_COUNT=0
[ -z "$TODO_COUNT" ] && TODO_COUNT=0

# Build state summary
STATE_SUMMARY=""
if [ "$BLOCKED_COUNT" != "0" ]; then
    STATE_SUMMARY="**${BLOCKED_COUNT} BLOCKED**, "
fi
if [ "$ACTIVE_COUNT" != "0" ]; then
    STATE_SUMMARY="${STATE_SUMMARY}${ACTIVE_COUNT} active, "
fi
if [ "$TODO_COUNT" != "0" ]; then
    STATE_SUMMARY="${STATE_SUMMARY}${TODO_COUNT} ready to start"
fi
STATE_SUMMARY="${STATE_SUMMARY:-No actionable items}"

# Build context message
read -r -d '' CONTEXT << EOF
## CONTEXT RESTORED: Cadre Project

### Planning Hierarchy
ProductDoc -> Epic -> Story -> Task

### CRITICAL: Work Tracking Rules
- **Do NOT use TodoWrite** for tracking work. Cadre documents ARE your work tracking system.
- **ALWAYS update active documents** with progress as you work.
- Check for active work with \`mcp__cadre__list_documents\`.

### Current Project State
${STATE_SUMMARY}

### Actionable Work Items
\`\`\`
${ACTIVE_WORK:-No active or ready items found}
\`\`\`

### MCP Tools
- \`mcp__cadre__list_documents\` - List all documents
- \`mcp__cadre__read_document\` - Read by short code (e.g., PROJ-E-0001)
- \`mcp__cadre__edit_document\` - Update document content
- \`mcp__cadre__transition_phase\` - Move through phases
- \`mcp__cadre__create_document\` - Create new documents (MUST populate content after!)
- \`mcp__cadre__reassign_parent\` - Move stories/tasks between parents

### Document Types: ProductDoc (PD), Epic (E), Story (S), Task (T), DesignContext (DC), ADR (A)
### Story Types: feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup

### CRITICAL: Creating Documents
After \`create_document\`, you MUST: \`read_document\` then \`edit_document\` to populate ALL content. Never leave placeholder text.

### CRITICAL: Human-in-the-Loop
For epics: ALWAYS check in with the human before phase transitions, design decisions, or decomposition. Present options and get approval.

### Story/Task Workflow
1. \`read_document\` - Understand requirements
2. \`transition_phase\` - Move to "active"
3. Work and update document with progress
4. \`transition_phase\` - Move to "completed"
EOF

# Output JSON for Claude - PreCompact uses systemContext field
cat << ENDJSON
{
    "systemContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
}
ENDJSON

exit 0
