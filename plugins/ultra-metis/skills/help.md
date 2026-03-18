---
name: ultra-metis:help
description: "Explain Ultra-Metis plugin and available commands. Use when user asks about ultra-metis, how it works, what tools are available, or needs an overview of the system."
---

# Ultra-Metis Help

Ultra-Metis is a repo-native AI engineering orchestration system. It manages work through a hierarchical document model that flows through lifecycle phases.

## Document Types

| Type | Purpose | Phases | Parent |
|------|---------|--------|--------|
| **ProductDoc** | Product definition anchoring all planning | draft → review → published | None |
| **Epic** | Major capability increments grouping stories | discovery → design → ready → decompose → active → completed | ProductDoc |
| **Story** | Typed implementation slices within epics | discovery → design → ready → active → completed (+ blocked) | Epic |
| **Task** | Execution-level work items | backlog → todo → active → completed (+ blocked) | Story |
| **DesignContext** | Approved UI patterns, design specs | draft → review → published → superseded | None |
| **ADR** | Architecture decisions | draft → discussion → decided → superseded | None |

### Story Types
Stories are typed by purpose: `feature`, `bugfix`, `refactor`, `migration`, `architecture-change`, `investigation`, `remediation`, `setup`

### The Planning Hierarchy
```
ProductDoc: "Why does this product exist?"
    ↓
Epic: "User Authentication" (capability increment)
    ↓
Story: "Login flow" (typed: feature)
    ↓
Task: "Implement OAuth callback handler" (execution unit)
```

## Available MCP Tools

All tools use the `mcp__ultra-metis__` prefix:

| Tool | Purpose |
|------|---------|
| `initialize_project` | Create a new Ultra-Metis workspace |
| `create_document` | Create product_doc, epic, story, task, design_context, or ADR |
| `read_document` | Read document content by short code |
| `edit_document` | Update document content (search-and-replace) |
| `list_documents` | List all documents with phases and short codes |
| `search_documents` | Full-text search across documents |
| `transition_phase` | Advance document to next phase |
| `archive_document` | Archive document and children |
| `reassign_parent` | Move task/story to different parent |
| `index_code` | Index source code symbols with tree-sitter |

## Common Workflows

### Start a New Project
1. `initialize_project` — create workspace with prefix
2. `create_document(type="product_doc")` — define product intent
3. `transition_phase` twice — draft → review → published
4. `create_document(type="epic")` — create capability increments under product doc

### Plan and Execute Work
1. Create Epic under published ProductDoc
2. Progress Epic through discovery → design → ready → decompose
3. Create Stories under Epic during decompose phase
4. Create Tasks under Stories for execution-level work
5. Execute tasks: todo → active → completed

## Short Codes

Every document gets a unique ID: `PREFIX-TYPE-NNNN`
- **PD** = ProductDoc, **E** = Epic, **S** = Story, **T** = Task, **DC** = DesignContext, **A** = ADR
- Example: `PROJ-E-0001`, `ACME-T-0042`

Use short codes to reference documents in all operations.

## Key Principles

- **Work is pulled, not pushed** — low backlog signals to decompose more
- **All work traces to product intent** — if it doesn't align, question its value
- **Phases exist for a reason** — don't skip them
- **Filesystem is truth** — documents are repo-native markdown+YAML
- **Scope over time** — size by capability, not duration
- **Static tools first** — prefer deterministic tools over unconstrained reasoning
