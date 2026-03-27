#!/bin/bash
# Cadre Ralph Setup Script
# Reads document via CLI, maps story type to superpowers skills,
# builds prompt, and creates ralph-loop state file.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check dependencies
"$SCRIPT_DIR/check-dependencies.sh"

# Parse arguments
SHORT_CODE=""
MAX_ITERATIONS=0
PROJECT_PATH=""

while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      cat << 'HELP_EOF'
Cadre Ralph - Execute a story or task with Ralph loop

USAGE:
  /cadre-ralph <SHORT_CODE> [OPTIONS]

ARGUMENTS:
  SHORT_CODE    Cadre short code (e.g., PROJ-S-0001 or PROJ-T-0001)

OPTIONS:
  --project-path <path>    Path to project root (default: auto-detect)
  --max-iterations <n>     Maximum iterations (default: unlimited)
  -h, --help               Show this help message

DESCRIPTION:
  Reads the document, determines story type, maps to superpowers skills,
  and starts a ralph-loop with a constructed prompt. Delegates iteration
  to the ralph-loop plugin and methodology to superpowers.
HELP_EOF
      exit 0
      ;;
    --max-iterations)
      if [[ -z "${2:-}" ]] || ! [[ "$2" =~ ^[0-9]+$ ]]; then
        echo "Error: --max-iterations requires a positive integer" >&2
        exit 1
      fi
      MAX_ITERATIONS="$2"
      shift 2
      ;;
    --project-path)
      if [[ -z "${2:-}" ]]; then
        echo "Error: --project-path requires a path" >&2
        exit 1
      fi
      PROJECT_PATH="$2"
      shift 2
      ;;
    -*)
      echo "Error: Unknown option: $1" >&2
      exit 1
      ;;
    *)
      if [[ -z "$SHORT_CODE" ]]; then
        SHORT_CODE="$1"
      else
        echo "Error: Unexpected argument: $1" >&2
        exit 1
      fi
      shift
      ;;
  esac
done

# Validate short code
if [[ -z "$SHORT_CODE" ]]; then
  echo "Error: No short code provided" >&2
  echo "Usage: /cadre-ralph <SHORT_CODE>" >&2
  exit 1
fi

# Validate short code format (Story: PREFIX-S-NNNN or Task: PREFIX-T-NNNN)
if ! [[ "$SHORT_CODE" =~ ^[A-Z]+-[ST]-[0-9]+$ ]]; then
  echo "Error: Invalid short code format: $SHORT_CODE" >&2
  echo "Expected: PREFIX-S-NNNN (story) or PREFIX-T-NNNN (task)" >&2
  echo "Use /cadre-decompose for epics (PREFIX-E-NNNN)" >&2
  exit 1
fi

# Auto-detect project path
if [[ -z "$PROJECT_PATH" ]]; then
  SEARCH_DIR="$(pwd)"
  while [[ "$SEARCH_DIR" != "/" ]]; do
    if [[ -d "$SEARCH_DIR/.metis" ]]; then
      PROJECT_PATH="$SEARCH_DIR"
      break
    fi
    SEARCH_DIR="$(dirname "$SEARCH_DIR")"
  done
  if [[ -z "$PROJECT_PATH" ]]; then
    echo "Error: Could not find .metis directory" >&2
    exit 1
  fi
fi

# Read document via CLI (deterministic)
DOC_RAW=$(cadre read "$SHORT_CODE" -p "$PROJECT_PATH" 2>&1) || {
  echo "Error: Could not read document $SHORT_CODE" >&2
  echo "$DOC_RAW" >&2
  exit 1
}

# Extract document type from frontmatter
DOC_TYPE=$(echo "$DOC_RAW" | grep -E "^level:" | head -1 | sed 's/level: *//' | tr -d '"' | tr -d ' ')

# Extract story_type if present
STORY_TYPE=$(echo "$DOC_RAW" | grep -E "^story_type:" | head -1 | sed 's/story_type: *//' | tr -d '"' | tr -d ' ')

# Map story type to superpowers skills (DETERMINISTIC)
SKILLS_INSTRUCTIONS=""
case "$STORY_TYPE" in
  feature)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:brainstorming - Explore the problem space before implementation
2. Invoke skill: superpowers:writing-plans - Create implementation plan from the story requirements
3. Invoke skill: superpowers:test-driven-development - Use TDD for all implementation
4. Invoke skill: superpowers:verification-before-completion - Verify work before claiming done"
    ;;
  bugfix)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:systematic-debugging - Follow systematic debugging methodology
2. Invoke skill: superpowers:verification-before-completion - Verify the fix before claiming done"
    ;;
  refactor)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:writing-plans - Plan the refactoring approach
2. Invoke skill: superpowers:verification-before-completion - Verify nothing is broken"
    ;;
  migration)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:writing-plans - Plan the migration steps
2. Invoke skill: superpowers:verification-before-completion - Verify migration completeness"
    ;;
  architecture-change|architecture_change)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:brainstorming - Explore architecture options
2. Invoke skill: superpowers:writing-plans - Plan the architecture change
3. Invoke skill: superpowers:verification-before-completion - Verify architecture integrity"
    ;;
  investigation)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:brainstorming - Explore the investigation space systematically"
    ;;
  remediation)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:systematic-debugging - Diagnose the root cause
2. Invoke skill: superpowers:verification-before-completion - Verify remediation is complete"
    ;;
  setup)
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:writing-plans - Plan the setup steps
2. Invoke skill: superpowers:verification-before-completion - Verify setup is complete"
    ;;
  *)
    # Task or unknown type - default skills
    SKILLS_INSTRUCTIONS="REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:test-driven-development - Use TDD for implementation
2. Invoke skill: superpowers:verification-before-completion - Verify work before claiming done"
    ;;
esac

# Build the full prompt
PROJECT_STATE_PATH="$PROJECT_PATH/.metis"
PROMPT="Execute Cadre document: $SHORT_CODE

## Document Content
$DOC_RAW

## $SKILLS_INSTRUCTIONS

## Execution Instructions

1. Read the document using mcp__cadre__read_document(project_path=\"$PROJECT_STATE_PATH\", short_code=\"$SHORT_CODE\") to get current state
2. Transition to active using mcp__cadre__transition_phase(project_path=\"$PROJECT_STATE_PATH\", short_code=\"$SHORT_CODE\")
3. Follow the REQUIRED SKILLS above - invoke each one using the Skill tool before starting implementation
4. Implement what the document describes
5. Log progress to the document using mcp__cadre__edit_document after each significant step
6. When FULLY complete:
   - Do NOT transition to completed (user will review)
   - Output: <promise>TASK COMPLETE</promise>

## Critical Rules
- ONLY output the promise when work is genuinely complete
- ALWAYS log progress to the document - it is your persistent working memory
- Do NOT lie or output false promises to escape the loop
- If stuck, continue iterating - the loop is designed for persistence"

# Create ralph-loop state file (uses ralph-loop plugin's infrastructure)
mkdir -p .claude

cat > .claude/ralph-loop.local.md <<EOF
---
active: true
iteration: 1
session_id: ${CLAUDE_SESSION_ID:-}
max_iterations: $MAX_ITERATIONS
completion_promise: "TASK COMPLETE"
started_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
---

$PROMPT
EOF

# Output setup message
SKILLS_LIST=$(echo "$SKILLS_INSTRUCTIONS" | grep "Invoke skill:" | sed 's/.*Invoke skill: /  - /' | head -5)

cat <<EOF
Cadre Ralph activated for: $SHORT_CODE

Document type: $DOC_TYPE
Story type: ${STORY_TYPE:-task}
Project: $PROJECT_PATH
Max iterations: $(if [[ $MAX_ITERATIONS -gt 0 ]]; then echo "$MAX_ITERATIONS"; else echo "unlimited"; fi)

Mapped superpowers skills:
$SKILLS_LIST

Iteration infrastructure: ralph-loop plugin (stop hook active)

To cancel: /cancel-ralph

$PROMPT
EOF
