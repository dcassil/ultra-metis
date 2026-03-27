---
id: set-up-external-catalog-repo-and
level: task
title: "Set Up External Catalog Repo and Migrate Builtin Entries"
short_code: "SMET-T-0212"
created_at: 2026-03-27T19:23:04.321976+00:00
updated_at: 2026-03-27T19:29:10.630591+00:00
parent: SMET-I-0097
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0097
---

# Set Up External Catalog Repo and Migrate Builtin Entries

## Parent Initiative

[[SMET-I-0097]]

## Objective

Create the `dcassil/cadre-architecture-docs` GitHub repo, migrate all existing builtin architecture catalog entries into it with proper organization, and set up CI validation. This establishes the external source of truth for the architecture catalog.

## Acceptance Criteria

## Acceptance Criteria

- [ ] GitHub repo `dcassil/cadre-architecture-docs` created and public
- [ ] All 9 existing builtin entries migrated: 5 JavaScript (server, react-app, component-lib, cli-tool, node-util) + 4 new (rust-cli-tool, rust-web-service, python-web-app, python-cli-tool)
- [ ] Entries organized as `{language}/{project-type}.md` (e.g., `javascript/server.md`, `rust/cli-tool.md`)
- [ ] YAML frontmatter matches the `ArchitectureCatalogEntry` serialization schema exactly
- [ ] README documents the schema, contribution guidelines, and how Cadre consumes the repo
- [ ] GitHub Actions CI validates frontmatter schema on every PR
- [ ] Each entry's markdown body contains the full content (overview, structure, dependency rules, anti-patterns, quality expectations)

## Implementation Notes

### Technical Approach

1. Use `gh repo create dcassil/cadre-architecture-docs --public` to create the repo
2. Convert each builtin entry to standalone markdown files with full YAML frontmatter:
   - Use the existing `ArchitectureCatalogEntry::to_content()` serialization as the canonical format
   - Each file should be self-contained and parseable by `ArchitectureCatalogEntry::from_content()`
3. Directory structure:
   ```
   javascript/
     server.md
     react-app.md
     component-lib.md
     cli-tool.md
     node-util.md
   rust/
     cli-tool.md
     web-service.md
   python/
     web-app.md
     cli-tool.md
   README.md
   .github/workflows/validate.yml
   ```
4. CI workflow: parse each `.md` file's frontmatter, validate required fields (title, language, project_type, folder_layout, layers, dependency_rules, naming_conventions, anti_patterns, rules_seed_hints, analysis_expectations)

### Source Files
- `crates/cadre-core/src/domain/catalog/builtin_data/` — all 9 `.md` files
- `crates/cadre-core/src/domain/catalog/builtin_entries.rs` — structured data for the 5 JS entries
- New Rust/Python entries in `builtin_data/` (untracked, need to check their frontmatter format)

### Dependencies
- GitHub CLI (`gh`) must be authenticated
- No code changes to cadre-core in this task — purely external repo setup

## Status Updates

### 2026-03-27
- Created GitHub repo `dcassil/cadre-architecture-docs` (was empty, already existed)
- Migrated all 9 architecture entries with full YAML frontmatter:
  - JavaScript: server, react-app, component-lib, cli-tool, node-util (data from builtin_entries.rs)
  - Rust: cli-tool, web-service (new structured data created)
  - Python: web-app, cli-tool (new structured data created)
- Organized as `{language}/{project-type}.md`
- Created README with schema docs, contribution guidelines, and entry table
- Created GitHub Actions CI workflow to validate frontmatter on PRs
- Pushed to main, CI validation workflow queued