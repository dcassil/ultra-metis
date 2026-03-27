---
id: evaluate-brownfield-and-contribute
level: task
title: "Evaluate Brownfield and Contribute Architecture Doc Plugin Skill"
short_code: "SMET-T-0216"
created_at: 2026-03-27T19:23:07.913206+00:00
updated_at: 2026-03-27T19:23:07.913206+00:00
parent: SMET-I-0097
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0097
---

# Evaluate Brownfield and Contribute Architecture Doc Plugin Skill

## Parent Initiative

[[SMET-I-0097]]

## Objective

Create a Cadre plugin skill that evaluates brownfield codebases and, when the architecture is high-quality and unique, generates an architecture catalog document and submits it as a PR to the public catalog repo. This enables community-driven catalog growth from real-world codebases.

## Acceptance Criteria

- [ ] New skill registered in the Cadre plugin
- [ ] Skill leverages the existing `evaluate_brownfield` MCP tool to analyze the target codebase
- [ ] Skill queries existing catalog entries to assess uniqueness of the detected architecture
- [ ] For high-quality (score >= 70) AND unique architectures: generates an architecture doc from the analysis
- [ ] Generated doc correctly maps brownfield analysis output (layers, boundaries, conventions, etc.) to the `ArchitectureCatalogEntry` schema
- [ ] Automatically submits the doc as a PR to `dcassil/cadre-architecture-docs` via `gh` CLI
- [ ] PR body includes: quality score, uniqueness assessment, source codebase context (anonymized if needed)
- [ ] For low-quality or non-unique architectures: reports findings to the user without submitting
- [ ] Skill provides clear feedback at each decision point (quality check, uniqueness check, PR submission)

## Implementation Notes

### Technical Approach

This is a **plugin skill** (prompt-based) that orchestrates existing MCP tools.

**Skill file:** `skills/contribute-brownfield-architecture.md`

**Skill flow:**
1. Accept input: path to the codebase to evaluate (defaults to current project)
2. Run `evaluate_brownfield` MCP tool against the codebase
3. Parse the evaluation result:
   - Extract `structure_quality_score`, detected layers, naming conventions, module boundaries, dependency patterns
   - Check the evaluation outcome type (CatalogMatch, DerivedArchitecture, RecommendCatalogPattern, RecordAsIs)
4. **Quality gate:** If `structure_quality_score < 70`, report findings and stop — architecture isn't mature enough to contribute
5. **Uniqueness gate:** Query `query_architecture_catalog` for the same language + detected project type
   - If an exact match exists, compare layers/boundaries/conventions for meaningful differences
   - If too similar (>80% overlap), report that a similar entry already exists and stop
6. **Generation:** If quality AND uniqueness pass, map the brownfield analysis to the full `ArchitectureCatalogEntry` schema:
   - `language`: from detection
   - `project_type`: derived from evaluation (e.g., "server", "cli-tool", or a new type)
   - `folder_layout`: from `StructureAnalysis.folder_structure`
   - `layers`: from `StructureAnalysis.detected_layers`
   - `module_boundaries`: from `StructureAnalysis.module_boundaries`
   - `dependency_rules`: inferred from layer relationships
   - `naming_conventions`: from `StructureAnalysis.naming_convention`
   - `anti_patterns`: common anti-patterns for the detected architecture style
   - `rules_seed_hints`: generated from the analysis
   - `analysis_expectations`: from quality findings
   - Markdown body: structured overview from the analysis
7. Save the generated doc locally for review
8. **PR submission:** Same flow as SMET-T-0215 — fork/branch/commit/PR via `gh` CLI
9. Report the PR URL and summary to the user

### Uniqueness Comparison Logic
Compare the candidate architecture against existing catalog entries:
- Layer overlap: if >80% of layers match an existing entry, it's a duplicate
- Folder structure similarity: compare top-level directories
- Project type: if exact match exists, require significant structural differences

### Skill Metadata
- **Name:** `contribute-brownfield-architecture`
- **Description:** Evaluate a brownfield codebase and contribute its architecture as a catalog entry if it's high-quality and unique
- **Trigger phrases:** "contribute architecture", "submit architecture to catalog", "brownfield contribute"

### Dependencies
- SMET-T-0212 (external repo must exist for PR submission)
- SMET-T-0214 (remote catalog must work for uniqueness checks)
- `evaluate_brownfield` MCP tool (existing)
- `query_architecture_catalog` MCP tool (existing)
- `gh` CLI (for PR submission)

## Status Updates

*To be added during implementation*