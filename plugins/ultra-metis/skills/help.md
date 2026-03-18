---
name: ultra-metis:help
description: "Explain Ultra-Metis plugin and available commands. Use when user asks about ultra-metis, how it works, what tools are available, or needs an overview of the system."
---

# Ultra-Metis Help

Ultra-Metis is a repo-native AI engineering orchestration system. It manages work through Flight Levels methodology — hierarchical documents that flow through lifecycle phases.

## Document Types

| Type | Purpose | Parent Required |
|------|---------|-----------------|
| **Vision** | Strategic direction (6mo-2yr) | No |
| **Strategy** | Coordinated approaches (Full preset only) | Vision (published) |
| **Initiative** | Concrete projects (1-6mo) | Strategy or Vision (published) |
| **Task** | Individual work (1-14 days) | Initiative (decompose/active) |
| **Backlog** | Standalone bugs/features/debt | No (use `backlog_category`) |
| **ADR** | Architecture decisions | No |

## Available MCP Tools

All tools use the `mcp__ultra-metis__` prefix:

| Tool | Purpose |
|------|---------|
| `initialize_project` | Create a new Ultra-Metis workspace |
| `create_document` | Create vision, strategy, initiative, task, or ADR |
| `read_document` | Read document content by short code |
| `edit_document` | Update document content (search-and-replace) |
| `list_documents` | List all documents with phases and short codes |
| `search_documents` | Full-text search across documents |
| `transition_phase` | Advance document to next phase |
| `archive_document` | Archive document and children |
| `reassign_parent` | Move task to different initiative or backlog |
| `index_code` | Index source code symbols with tree-sitter |

## Common Workflows

### Start a New Project
1. `initialize_project` — create workspace with prefix
2. `create_document(type="vision")` — define strategic direction
3. `transition_phase` twice — draft to review to published
4. `create_document(type="initiative")` — create work under vision

### Track Work
1. `list_documents` — see all active work
2. `read_document` — check document details
3. `edit_document` — update progress, add notes
4. `transition_phase` — advance through lifecycle

### Execute a Task
1. `read_document` — understand the task
2. `transition_phase` — move to active
3. Do the work, updating progress via `edit_document`
4. `transition_phase` — move to completed

### Create Backlog Items
For standalone bugs, features, or tech debt:
```
create_document(type="task", title="Fix login timeout", backlog_category="bug")
```

### Decompose an Initiative
1. Transition initiative to "decompose" phase
2. Create tasks with `parent_id` pointing to the initiative
3. Transition initiative to "active" when ready

## Short Codes

Every document gets a unique ID: `PREFIX-TYPE-NNNN`
- **V** = Vision, **S** = Strategy, **I** = Initiative, **T** = Task, **A** = ADR
- Example: `PROJ-I-0001`, `ACME-T-0042`

Use short codes to reference documents in all operations.

## Presets

| Preset | Hierarchy | Best For |
|--------|-----------|----------|
| **Direct** | Vision → Task | Solo work |
| **Streamlined** | Vision → Initiative → Task | Most projects (default) |
| **Full** | Vision → Strategy → Initiative → Task | Multi-team coordination |

## Key Principles

- **Work is pulled, not pushed** — low backlog signals to decompose more
- **All work traces to vision** — if it doesn't align, question its value
- **Phases exist for a reason** — don't skip them
- **Filesystem is truth** — documents are repo-native markdown+YAML
- **Scope over time** — size by capability, not duration
