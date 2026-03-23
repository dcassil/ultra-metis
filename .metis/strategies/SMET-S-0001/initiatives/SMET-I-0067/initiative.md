---
id: cadre-plugin-execution
level: initiative
title: "Cadre Plugin: Execution Commands (Ralph and Decompose)"
short_code: "SMET-I-0067"
created_at: 2026-03-18T17:50:12.095727+00:00
updated_at: 2026-03-18T18:48:48.369289+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: cadre-plugin-execution
---

# Cadre Plugin: Execution Commands (Ralph and Decompose)

## Strategy Update (2026-03-18)

**Revised approach**: Compose existing plugins instead of building custom infrastructure.
- **ralph-loop plugin** (Anthropic official) — provides the iteration mechanism (stop hook, state file, max-iterations, completion promises)
- **superpowers plugin** — provides execution methodology (TDD, debugging, verification, planning)
- **cadre commands** — thin deterministic wrappers that read documents via CLI, map story types to skills, construct prompts, and invoke ralph-loop

**Key principle**: Move as much as possible from AI reasoning to deterministic tools. Shell scripts read documents, map types to skills, and construct prompts. AI executes the work using explicitly-invoked skills.

**Plugin dependencies**: ralph-loop and superpowers must be installed. Commands fail fast with clear error if missing.

## Context

Cadre needs execution commands that convert documents into actionable work loops. Rather than building custom loop infrastructure, we compose:
- `cadre` CLI for document reading (deterministic)
- ralph-loop plugin for iteration (deterministic stop hook + state file)
- superpowers plugin for methodology (explicit skill invocation)

The commands are shell scripts + command .md files. Shell scripts handle all deterministic work; command .md files wire up the Claude Code integration.

## Goals & Non-Goals

**Goals:**
- Create `/cadre-ralph <short-code>` command for story/task execution via ralph-loop
- Create `/cadre-decompose <short-code>` command for epic decomposition
- Create `/cadre-ralph-epic <short-code>` command for multi-story execution
- Create `/cancel-cadre-ralph` command for canceling loops
- Shell scripts read documents via `cadre` CLI (deterministic)
- Story type → superpowers skill mapping hardcoded in shell script (deterministic)
- Direct invocation of ralph-loop setup script (not reimplemented)
- Direct invocation of superpowers skills via Skill tool (not context-triggered)
- Fail fast if ralph-loop or superpowers plugins not installed

**Non-Goals:**
- Building custom loop infrastructure (use ralph-loop)
- Building custom methodology (use superpowers)
- Building Rust execution engine
- AI-decided skill selection (mapping is deterministic)

## Detailed Design

### Plugin Directory Structure
```
plugins/cadre/
  commands/
    cadre-ralph.md          # Command: execute story/task
    cadre-decompose.md      # Command: decompose epic
    cadre-ralph-epic.md     # Command: execute all stories in epic
    cancel-cadre-ralph.md   # Command: cancel active loop
  scripts/
    setup-cadre-ralph.sh    # Shell: read doc, map type→skills, build prompt, invoke ralph-loop
    setup-cadre-decompose.sh # Shell: read epic, build decomposition prompt
    check-dependencies.sh         # Shell: verify ralph-loop + superpowers installed
```

### Story Type → Skill Mapping (Deterministic)

Shell script maps `story_type` to required superpowers skills:

| Story Type | Superpowers Skills |
|------------|-------------------|
| feature | brainstorming, writing-plans, test-driven-development, verification-before-completion |
| bugfix | systematic-debugging, verification-before-completion |
| refactor | writing-plans, verification-before-completion |
| migration | writing-plans, verification-before-completion |
| architecture-change | brainstorming, writing-plans, verification-before-completion |
| investigation | brainstorming |
| remediation | systematic-debugging, verification-before-completion |
| setup | writing-plans, verification-before-completion |

For tasks (no story type): test-driven-development, verification-before-completion

### `/cadre-ralph <short-code>` Flow

**Shell script** (deterministic):
1. Check ralph-loop + superpowers installed (`check-dependencies.sh`)
2. `cadre read <short-code> --json` → get document type, title, content, story_type, acceptance criteria
3. Validate document is a Story or Task (reject other types)
4. Lookup story_type → required superpowers skills
5. Build prompt including:
   - Document content and acceptance criteria
   - Explicit Skill tool invocations for each mapped skill
   - Instructions to update progress via `mcp__cadre__edit_document`
   - Completion promise: "TASK COMPLETE"
6. `cadre transition <short-code>` → move to active phase
7. Invoke ralph-loop setup script with constructed prompt + `--completion-promise "TASK COMPLETE"`

**Command .md**: Bash execution block that runs the shell script, plus post-loop instructions to transition to completed.

### `/cadre-decompose <short-code>` Flow

**Shell script** (deterministic):
1. Check superpowers installed
2. `cadre read <short-code> --json` → get epic content
3. Validate document is an Epic
4. Build prompt including:
   - Epic content, goals, acceptance criteria
   - Explicit invocation of `cadre:decomposition` skill
   - Explicit invocation of `superpowers:brainstorming` skill
   - Instructions to create stories via `mcp__cadre__create_document` with story_type
   - Instructions to populate each story via `mcp__cadre__edit_document`
   - Human review gate before creating stories
   - Completion promise: "DECOMPOSITION COMPLETE"
5. `cadre transition <short-code>` → move to decompose phase
6. Invoke ralph-loop setup script with prompt + `--completion-promise "DECOMPOSITION COMPLETE"`

### `/cancel-cadre-ralph`

Delegates directly to `/cancel-ralph` from ralph-loop plugin. Additionally updates the active cadre document with cancellation note.

### Dependency Checking

`check-dependencies.sh`:
```bash
# Check ralph-loop plugin
if [ ! -f ".claude/ralph-loop.local.md" ] && ! command -v ralph-loop &>/dev/null; then
  # Check for ralph-loop stop hook in plugin cache
  if ! find ~/.claude/plugins -path "*/ralph-loop/hooks/stop-hook.sh" -print -quit 2>/dev/null | grep -q .; then
    echo "ERROR: ralph-loop plugin not installed. Run: claude plugin add ralph-loop@claude-plugins-official"
    exit 1
  fi
fi
# Check superpowers plugin  
if ! find ~/.claude/plugins -path "*/superpowers/*" -print -quit 2>/dev/null | grep -q .; then
  echo "ERROR: superpowers plugin not installed. Run: claude plugin add superpowers@claude-plugins-official"
  exit 1
fi
```

## Alternatives Considered

**Alternative 1: Build custom loop infrastructure**
- Rejected: ralph-loop plugin already handles iteration, state, stop hook, max-iterations, completion promises. Don't rebuild.

**Alternative 2: AI-decided skill selection (hope skills trigger from context)**
- Rejected: Deterministic mapping is more reliable. Shell script maps story_type → skills, prompt explicitly invokes them.

**Alternative 3: Pure .md commands with no shell scripts**
- Rejected: Shell scripts move document reading, type mapping, and prompt construction from AI to deterministic tools. This is the core design principle.

## Dependencies
- **ralph-loop plugin** (claude-plugins-official) — loop infrastructure
- **superpowers plugin** (claude-plugins-official) — execution methodology
- SMET-I-0064 (Skills) — decomposition guidance content
- SMET-I-0066 (Hooks) — session start context
- `cadre` CLI must be installed and on PATH

## Implementation Plan

1. Create `scripts/check-dependencies.sh` — verify required plugins
2. Create `scripts/setup-cadre-ralph.sh` — read doc, map type→skills, build prompt, invoke ralph-loop
3. Create `commands/cadre-ralph.md` — command wrapper
4. Create `scripts/setup-cadre-decompose.sh` — read epic, build decomposition prompt
5. Create `commands/cadre-decompose.md` — command wrapper
6. Create `commands/cadre-ralph-epic.md` — multi-story execution
7. Create `commands/cancel-cadre-ralph.md` — cancel wrapper
8. Test with real documents end-to-end

## Progress

### 2026-03-18 — Commands and Scripts Created
Created in `plugins/cadre/`:

**Scripts (deterministic):**
- `scripts/check-dependencies.sh` — verifies ralph-loop, superpowers, and cadre CLI installed
- `scripts/setup-cadre-ralph.sh` — reads doc via CLI, extracts story_type, maps to superpowers skills, builds prompt, creates ralph-loop state file
- `scripts/setup-cadre-decompose.sh` — reads epic via CLI, builds decomposition prompt with brainstorming + decomposition skills, creates ralph-loop state file

**Commands (.md wrappers):**
- `commands/cadre-ralph.md` — `/cadre-ralph <SHORT_CODE>` for story/task execution
- `commands/cadre-decompose.md` — `/cadre-decompose <SHORT_CODE>` for epic decomposition
- `commands/cadre-ralph-epic.md` — `/cadre-ralph-epic <SHORT_CODE>` for sequential multi-story execution
- `commands/cancel-cadre-ralph.md` — delegates to `ralph-loop:cancel-ralph`

**Key architecture decisions:**
- Shell scripts do ALL document reading, type→skill mapping, and prompt construction (deterministic)
- ralph-loop plugin's state file format (`.claude/ralph-loop.local.md`) used directly — no custom loop infrastructure
- Superpowers skills explicitly invoked via Skill tool in constructed prompts, not context-triggered
- 8 story types deterministically mapped to specific superpowers skill combinations

## Exit Criteria
- [x] All commands created with shell scripts + command .md files
- [x] `check-dependencies.sh` verifies ralph-loop and superpowers are installed
- [x] Story type → skill mapping is deterministic (no AI decision)
- [x] ralph-loop handles iteration (no custom loop infrastructure)
- [x] Superpowers skills explicitly invoked (not context-triggered)
- [x] Document reading and prompt construction done in shell (not AI)
- [x] Phase transitions automated (active on start via script)