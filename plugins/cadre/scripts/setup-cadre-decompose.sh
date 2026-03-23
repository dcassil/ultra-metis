#!/bin/bash
# Cadre Decompose Setup Script
# Reads epic via CLI, builds decomposition prompt with skill invocations,
# and creates ralph-loop state file.

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
Cadre Decompose - Break an epic into typed stories

USAGE:
  /cadre-decompose <SHORT_CODE> [OPTIONS]

ARGUMENTS:
  SHORT_CODE    Cadre epic short code (e.g., PROJ-E-0001)

OPTIONS:
  --project-path <path>    Path to project root (default: auto-detect)
  --max-iterations <n>     Maximum iterations (default: unlimited)
  -h, --help               Show this help message

DESCRIPTION:
  Reads the epic, invokes brainstorming and decomposition skills,
  and starts a ralph-loop for structured decomposition into typed stories.
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
  echo "Error: No epic short code provided" >&2
  echo "Usage: /cadre-decompose <SHORT_CODE>" >&2
  exit 1
fi

# Validate short code format (Epic: PREFIX-E-NNNN)
if ! [[ "$SHORT_CODE" =~ ^[A-Z]+-E-[0-9]+$ ]]; then
  echo "Error: Invalid epic short code format: $SHORT_CODE" >&2
  echo "Expected: PREFIX-E-NNNN (e.g., PROJ-E-0001)" >&2
  echo "Use /cadre-ralph for stories (PREFIX-S-NNNN) or tasks (PREFIX-T-NNNN)" >&2
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

# Read epic via CLI (deterministic)
DOC_RAW=$(cadre read "$SHORT_CODE" -p "$PROJECT_PATH" 2>&1) || {
  echo "Error: Could not read document $SHORT_CODE" >&2
  echo "$DOC_RAW" >&2
  exit 1
}

# Build decomposition prompt
METIS_PATH="$PROJECT_PATH/.metis"
PROMPT="Decompose Cadre epic: $SHORT_CODE

## Epic Content
$DOC_RAW

## REQUIRED SKILLS (invoke each using the Skill tool):
1. Invoke skill: superpowers:brainstorming - Explore the problem space and identify stories
2. Invoke skill: cadre:decomposition - Follow decomposition methodology for story breakdown

## Decomposition Instructions

1. Read the epic using mcp__cadre__read_document(project_path=\"$METIS_PATH\", short_code=\"$SHORT_CODE\")
2. Transition to decompose using mcp__cadre__transition_phase(project_path=\"$METIS_PATH\", short_code=\"$SHORT_CODE\")
3. Invoke the REQUIRED SKILLS above using the Skill tool
4. Analyze the epic requirements and break into typed stories
5. Present the proposed story breakdown to the human for review BEFORE creating
6. After human approval, create each story using mcp__cadre__create_document:
   - type=\"story\"
   - parent_id=\"$SHORT_CODE\"
   - story_type=<appropriate type: feature, bugfix, refactor, migration, architecture-change, investigation, remediation, setup>
7. IMMEDIATELY populate each story with real content using mcp__cadre__edit_document:
   - Clear description of what to build/fix/change
   - Acceptance criteria
   - Technical notes and constraints
   - A document with template placeholders is INCOMPLETE
8. Log progress to the epic document using mcp__cadre__edit_document

## Story Quality Checklist
Each story must be:
- Independently valuable (delivers something useful alone)
- Clearly scoped (know when it's done)
- Properly typed (feature, bugfix, refactor, etc.)
- Aligned to the epic (contributes to parent goals)

## Decomposition Patterns (prefer vertical slices)
- Vertical slices: Break by user-visible functionality
- Risk-first: Address uncertain work first (investigation stories)
- Milestone-based: Break by deliverable checkpoints

## Completion
When decomposition is FULLY complete:
- Do NOT transition the epic to active (user will review)
- Output: <promise>DECOMPOSITION COMPLETE</promise>

## Critical Rules
- ALWAYS get human approval before creating stories
- ALWAYS populate story content immediately after creation
- ALWAYS set story_type for each story
- Do NOT create overly granular stories
- Do NOT lie or output false promises"

# Create ralph-loop state file
mkdir -p .claude

cat > .claude/ralph-loop.local.md <<EOF
---
active: true
iteration: 1
session_id: ${CLAUDE_SESSION_ID:-}
max_iterations: $MAX_ITERATIONS
completion_promise: "DECOMPOSITION COMPLETE"
started_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
---

$PROMPT
EOF

# Output setup message
cat <<EOF
Cadre Decompose activated for: $SHORT_CODE

Project: $PROJECT_PATH
Max iterations: $(if [[ $MAX_ITERATIONS -gt 0 ]]; then echo $MAX_ITERATIONS; else echo "unlimited"; fi)

Mapped skills:
  - superpowers:brainstorming
  - cadre:decomposition

Iteration infrastructure: ralph-loop plugin (stop hook active)

To cancel: /cancel-ralph

$PROMPT
EOF
