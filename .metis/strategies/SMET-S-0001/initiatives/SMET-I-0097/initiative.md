---
id: externalize-architecture-catalog
level: initiative
title: "Externalize Architecture Catalog to Public Repo with Community Contribution Skills"
short_code: "SMET-I-0097"
created_at: 2026-03-27T18:36:07.181157+00:00
updated_at: 2026-03-27T20:30:07.991726+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: SMET-S-0001
initiative_id: externalize-architecture-catalog
---

# Externalize Architecture Catalog to Public Repo with Community Contribution Skills Initiative

## Context

The architecture catalog currently lives as builtin data inside `crates/cadre-core/src/domain/catalog/builtin_data/` — hardcoded markdown files for React, Node API, etc. This couples catalog content to the Cadre release cycle, limits community contributions, and makes the catalog difficult to extend without rebuilding the binary.

This initiative moves the catalog to a dedicated public repo (`https://github.com/dcassil/cadre-architecture-docs`) and adds Cadre plugin skills that:
1. Generate new architecture docs from industry best practices given a language and project type
2. Evaluate brownfield codebases and, if the architecture is high-quality and unique, generate an architecture doc and submit it as a PR to the public repo
3. Fetch catalog entries at runtime from the external repo instead of embedding them

## Goals & Non-Goals

**Goals:**
- Extract all builtin architecture catalog entries from cadre-core to `dcassil/cadre-architecture-docs`
- Update cadre-core to fetch/load catalog entries from the external repo (with local caching)
- Create a plugin skill to generate architecture docs from industry best practices given a language + project type/function
- Create a plugin skill (or enhance existing) to evaluate brownfield codebases and produce architecture docs from high-quality, unique architectures
- Auto-submit PRs to `dcassil/cadre-architecture-docs` for community-contributed architecture docs
- Existing evaluate_brownfield and query_architecture_catalog MCP tools continue working seamlessly

**Non-Goals:**
- Building a web UI for browsing architecture docs (the repo itself is the UI)
- Supporting private/enterprise catalog repos (public only for now)
- Changing the architecture doc format — reuse existing markdown + YAML frontmatter structure

## Detailed Design

### 1. External Repo Structure (`dcassil/cadre-architecture-docs`)
- One markdown file per architecture entry, organized by `language/project-type.md`
- YAML frontmatter matching the existing `ArchitectureCatalogEntry` schema
- README with contribution guidelines and schema documentation
- CI that validates frontmatter schema on PRs

### 2. Cadre-Core Changes
- Remove builtin_data directory contents (keep the module for local/custom entries)
- Add a catalog fetcher that clones/pulls the external repo into a local cache directory
- Catalog query engine reads from local cache + any custom entries in `.cadre/`
- Fallback: if offline/no cache, gracefully degrade with empty catalog

### 3. New Plugin Skills

**Skill: Create Architecture Doc from Best Practices**
- Input: language, project type/function (e.g., "Rust", "CLI tool")
- Behavior: Researches industry-standard best practices for the given combination, generates an architecture doc following the catalog schema
- Output: Creates the doc locally, optionally submits as PR to the public repo
- Should check existing catalog for uniqueness before submitting

**Skill: Evaluate Brownfield and Contribute Architecture Doc**
- Leverages existing `evaluate_brownfield` MCP tool
- If the evaluated architecture scores high on quality metrics AND is unique from existing catalog entries:
  - Generates an architecture doc from the evaluated codebase
  - Submits as a PR to `dcassil/cadre-architecture-docs`
- If not unique enough or quality is low, reports findings but does not submit

### 4. PR Submission Flow
- Uses `gh` CLI for PR creation (assumes user has GitHub auth configured)
- Forks the repo if needed, creates a branch, commits the doc, opens PR
- PR body includes quality score, uniqueness assessment, and source context

## Alternatives Considered

- **Keep catalog embedded**: Simpler but limits community contribution and couples content to release cycle. Rejected because the catalog should grow independently.
- **NPM/crates.io package for catalog**: Over-engineered for markdown files. A git repo is simpler and more accessible.
- **Allow private catalog repos**: Adds complexity (auth, config). Can be added later if needed.

## Implementation Plan

1. Set up `dcassil/cadre-architecture-docs` repo with existing builtin entries and CI
2. Update cadre-core catalog module to fetch from external repo with local caching
3. Remove builtin_data contents from cadre-core, verify existing tools still work
4. Create "generate architecture doc from best practices" plugin skill
5. Create/enhance "evaluate brownfield and contribute" plugin skill with PR submission
6. End-to-end testing: generate doc, evaluate brownfield, submit PR flow