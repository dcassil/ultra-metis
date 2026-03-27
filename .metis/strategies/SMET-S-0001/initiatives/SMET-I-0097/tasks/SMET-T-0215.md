---
id: generate-architecture-doc-from
level: task
title: "Generate Architecture Doc from Best Practices Plugin Skill"
short_code: "SMET-T-0215"
created_at: 2026-03-27T19:23:07.340033+00:00
updated_at: 2026-03-27T20:21:29.029299+00:00
parent: SMET-I-0097
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0097
---

# Generate Architecture Doc from Best Practices Plugin Skill

## Parent Initiative

[[SMET-I-0097]]

## Objective

Create a Cadre plugin skill that generates high-quality architecture catalog documents from industry best practices, given a language and project type. The skill guides Claude through researching patterns, generating the document in the correct schema, and optionally submitting it as a PR to the public catalog repo.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New skill registered in the Cadre plugin (or a new companion plugin)
- [ ] Skill accepts language + project type/function as input (e.g., "Go", "microservice")
- [ ] Generated architecture doc follows the `ArchitectureCatalogEntry` YAML frontmatter schema exactly
- [ ] Generated doc includes all required fields: folder_layout, layers, module_boundaries, dependency_rules, naming_conventions, anti_patterns, rules_seed_hints, analysis_expectations
- [ ] Markdown body includes: Overview, Structure, Dependency Rules, Anti-Patterns, Quality Expectations sections
- [ ] Skill checks existing catalog entries for uniqueness before proposing submission
- [ ] Skill can save the doc locally to a specified path
- [ ] Skill can optionally submit the doc as a PR to `dcassil/cadre-architecture-docs` using `gh` CLI
- [ ] PR body includes context about the generation source and methodology

## Implementation Notes

### Technical Approach

This is a **plugin skill** (prompt-based), not compiled Rust code. It lives as a markdown file in the plugin's `skills/` directory.

**Skill file:** `skills/generate-architecture-doc.md`

**Skill flow:**
1. Accept input: language, project type/function description
2. Query existing catalog entries via `query_architecture_catalog` MCP tool to check for duplicates
3. If a similar entry exists, inform the user and ask whether to proceed (may be creating a variant)
4. Research industry best practices for the language + project type combination:
   - Standard directory structures
   - Architectural layers and their responsibilities
   - Dependency flow rules
   - Naming conventions
   - Common anti-patterns
   - Quality/lint expectations
5. Generate the architecture doc with full YAML frontmatter + markdown body
6. Save locally (user specifies path, or default to current directory)
7. Optionally: fork/branch/commit/PR to `dcassil/cadre-architecture-docs`

**PR submission sub-flow (bash within skill):**
```bash
# Clone or update the repo
gh repo clone dcassil/cadre-architecture-docs /tmp/cadre-arch-docs || git -C /tmp/cadre-arch-docs pull
# Create branch
git -C /tmp/cadre-arch-docs checkout -b add-{language}-{project-type}
# Copy the generated doc
cp <generated-file> /tmp/cadre-arch-docs/{language}/{project-type}.md
# Commit and push
git -C /tmp/cadre-arch-docs add . && git -C /tmp/cadre-arch-docs commit -m "Add {language} {project-type} architecture doc"
git -C /tmp/cadre-arch-docs push -u origin add-{language}-{project-type}
# Create PR
gh pr create --repo dcassil/cadre-architecture-docs --title "Add {language} {project-type} architecture doc" --body "..."
```

### Skill Metadata
- **Name:** `generate-architecture-doc`
- **Description:** Generate an architecture catalog document from industry best practices for a given language and project type
- **Trigger phrases:** "generate architecture doc", "create catalog entry", "new architecture pattern"

### Dependencies
- SMET-T-0212 (external repo must exist for PR submission)
- `query_architecture_catalog` MCP tool (for uniqueness check)
- `gh` CLI (for PR submission)

## Status Updates

### 2026-03-27
- Created `plugins/cadre/skills/generate-architecture-doc/SKILL.md`
- Skill accepts language + project type, checks for duplicates via `query_architecture_catalog`
- Generates full YAML frontmatter matching `ArchitectureCatalogEntry` schema exactly
- Markdown body includes Overview, Structure, Dependency Rules, Anti-Patterns, Quality Expectations
- Saves locally to user-specified path or default
- Optional PR submission flow using `gh` CLI to `dcassil/cadre-architecture-docs`
- Trigger description covers: "generate architecture doc", "create catalog entry", "new architecture pattern", "add architecture to catalog"
- Plugin uses auto-discovery — skill registered automatically