# Cadre Project Rules

## Metis Workflow (MANDATORY)

This project uses **Metis Flight Levels** as the primary work management system. All work MUST flow through Metis documents — do NOT use TodoWrite or ad-hoc task tracking.

### Initiative Workflow (MUST follow in entirety)

When working on any initiative, you MUST follow every phase in order. **No skipping phases. No autonomous transitions.**

1. **Discovery** — Read the initiative fully. Ask clarifying questions about scope, priorities, and constraints. Do NOT assume you understand the full context. Present findings and get human confirmation before proceeding.

2. **Design** — Present multiple design options with trade-offs. Let the human choose the approach. Update the initiative document with the chosen design. Get explicit approval before advancing.

3. **Ready** — Confirm all prerequisites are met, dependencies are satisfied, and the design is approved. Summarize readiness and get human approval to proceed.

4. **Decompose** — Use `/metis-decompose <short-code>` to break the initiative into tasks. Review the proposed task breakdown with the human. Get explicit approval before creating tasks. Each task should be a vertical slice (1-14 days of work).

5. **Active** — Execute tasks using `/metis-ralph <short-code>`. Update active tasks with progress regularly. Tasks flow: todo → active → completed.

6. **Completed** — All tasks done, validations passed, acceptance criteria met. Summarize what was accomplished and get human confirmation.

### Phase Transition Rules

- **ALWAYS check in with the human** before transitioning any initiative phase
- Present: current state, what was accomplished, what the next phase entails
- Get **explicit approval** before calling `transition_phase`
- Never autonomously transition initiatives through multiple phases

### Task Execution

- Use `/metis-ralph <short-code>` for executing individual tasks
- Update active tasks with progress after each significant step
- Tasks serve as persistent working memory — record findings, decisions, and plan changes
- Transition tasks: todo → active (start) → completed (finish)

### Active Document Updates

While working on any active task or initiative:
- Record progress after completing each significant step
- Document unexpected discoveries and blockers
- Record decisions and why alternatives were rejected
- Update next steps if the plan changes
- This ensures no work is lost across sessions

## Plugin Usage

### Required Plugins
- **Metis** — Primary work management. All work tracked here.
- **Superpowers** — Use brainstorming before creative work. Use TDD for implementation. Use systematic debugging for bugs. Use verification before claiming completion.
- **Ralph Loop** — Use `/metis-ralph` for task execution with iterative loops.
- **Code Review** — Use for quality verification at completion gates.
- **Claude MD Management** — For syncing and maintaining project rules.

### Plugin Integration with Metis Phases
- **Discovery**: Use superpowers:brainstorming to explore the problem space
- **Design**: Use superpowers:writing-plans for implementation planning
- **Decompose**: Use `/metis-decompose` for structured task breakdown
- **Active**: Use `/metis-ralph` for task execution, code-review for quality checks
- **Completed**: Use superpowers:verification-before-completion, then code-review for final review

## Project Context

- **Vision**: SMET-V-0001 — Super-Metis: Repo-Native AI Engineering Orchestration
- **Metis path**: `/Users/danielcassil/projects/ultra-metis/.metis`
- **Preset**: Streamlined (Vision → Initiative → Task)
- **Crates location**: `crates/` at repo root (cadre-core, cadre-cli, cadre-mcp, cadre-store)
- **Workspace root**: `Cargo.toml` at repo root
- **Language**: Rust

## Building & Installing

```bash
make build          # Build all release binaries
make install        # Build + copy to ~/.local/bin
make test           # Run all workspace tests
```

After `make install`, the `cadre-mcp` and `cadre` binaries are on PATH. The `.mcp.json` at the repo root configures Claude Code to use the MCP server for this project.

## Code Standards

- Follow existing patterns in `crates/cadre-core/` codebase
- Strong Rust typing — prefer compile-time guarantees
- All durable state is repo-native (file-based, no external services)
- Markdown + YAML frontmatter for document serialization
- Comprehensive unit tests for domain types and serialization
