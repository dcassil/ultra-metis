#!/bin/bash
# SessionStart hook for Cadre projects
# Detects .metis directory and provides comprehensive project context

# Read hook input from stdin to extract session_id
HOOK_INPUT=$(cat)

# Export session ID so Bash tool commands can use it for session-scoped state files
SESSION_ID=$(echo "$HOOK_INPUT" | jq -r '.session_id // empty')
if [ -n "$CLAUDE_ENV_FILE" ] && [ -n "$SESSION_ID" ]; then
    echo "export CLAUDE_SESSION_ID='$SESSION_ID'" >> "$CLAUDE_ENV_FILE"
fi

# Exit silently if not in an Cadre project
if [ ! -d "$CLAUDE_PROJECT_DIR/.metis" ]; then
    exit 0
fi

# Check if cadre is installed
if ! command -v cadre &> /dev/null; then
    cat << 'ENDJSON'
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "WARNING: This is an Cadre project (`.metis` directory found) but the `cadre` command is not installed or not in PATH. Run `make install` from the cadre repo root."
    }
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
# Ensure counts are numbers
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
This is an **Cadre project** (detected \`.metis\` directory).

## Planning Hierarchy
ProductDoc -> Epic -> Story -> Task

## CRITICAL: Work Tracking Rules
- **Do NOT use TodoWrite** for tracking work in this project. Cadre documents ARE your work tracking system.
- **ALWAYS update active documents** with progress as you work - they serve as persistent memory across sessions.
- Before starting work, check for active work with \`mcp__cadre__list_documents\`.

## Current Project State
${STATE_SUMMARY}

### Actionable Work Items
\`\`\`
${ACTIVE_WORK:-No active or ready items found}
\`\`\`

## MCP Tools (Preferred)
Use these MCP tools for all Cadre operations:
- \`mcp__cadre__list_documents\` - List all documents with short codes and phases
- \`mcp__cadre__read_document\` - Read a document by short code (e.g., PROJ-E-0001)
- \`mcp__cadre__edit_document\` - Update document content (search and replace)
- \`mcp__cadre__transition_phase\` - Move documents through phases
- \`mcp__cadre__create_document\` - Create product_doc, epic, story, task, design_context, or ADR
- \`mcp__cadre__reassign_parent\` - Move stories/tasks between parents
- \`mcp__cadre__search_documents\` - Full-text search across documents
- \`mcp__cadre__archive_document\` - Archive document and children
- \`mcp__cadre__index_code\` - Index source code symbols

## Document Types & Phases
| Type | Phases | Short Code |
|------|--------|------------|
| ProductDoc | draft -> review -> published | PD |
| Epic | discovery -> design -> ready -> decompose -> active -> completed | E |
| Story | discovery -> design -> ready -> active -> completed (+ blocked) | S |
| Task | backlog -> todo -> active -> completed (+ blocked) | T |
| DesignContext | draft -> review -> published -> superseded | DC |
| ADR | draft -> discussion -> decided -> superseded | A |

### Story Types
feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup

## CRITICAL: Creating Documents
When you create a document, you MUST immediately populate it with content:
1. \`mcp__cadre__create_document\` - Creates document with template
2. \`mcp__cadre__read_document\` - Read the template structure
3. \`mcp__cadre__edit_document\` - Replace ALL placeholders with real content

**A document with template placeholders is INCOMPLETE. Never leave {placeholder} text.**

## CRITICAL: Human-in-the-Loop for Epics
For epics, you MUST check in with the human before:
- Transitioning to a new phase
- Making design/architectural decisions
- Decomposing into stories
- Any significant directional choice

Present options, ask clarifying questions, and get explicit approval. Do NOT proceed autonomously on strategic work.

## Working on a Story/Task
1. \`mcp__cadre__read_document\` - Read the document to understand requirements
2. \`mcp__cadre__transition_phase\` - Transition to "active"
3. Work on it, updating the document with progress regularly
4. \`mcp__cadre__transition_phase\` - Transition to "completed" when done
EOF

# Output JSON for Claude
cat << ENDJSON
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
    }
}
ENDJSON

exit 0
