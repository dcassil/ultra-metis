//! System prompt for the Cadre MCP server.
//!
//! Provides inline documentation for agents using the MCP tools.

/// The system prompt content, kept under ~4000 tokens.
pub const SYSTEM_PROMPT: &str = r"# Cadre MCP Server

Repo-native engineering orchestration. All state is file-based markdown+YAML frontmatter.

## Document Types & Phases

| Type | Prefix | Phases | Parent |
|------|--------|--------|--------|
| vision | V | draft > review > published | none |
| initiative | I | discovery > design > ready > decompose > active > completed | Vision |
| task | T | backlog > todo > active > completed | Initiative/Epic/Story |
| analysis_baseline | AB | draft > review > published | none |
| quality_record | QR | draft > review > published | none |
| rules_config | RC | draft > review > published | none |
| durable_insight_note | DIN | draft > review > published | none |
| cross_reference | XR | draft > review > published | none |
| architecture_catalog_entry | ACE | draft > review > published | none |
| reference_architecture | RA | draft > review > published | none |

## Core Document Tools

- **create_document** — Create vision/initiative/task/governance docs. Returns short code.
- **read_document** — Read full document content by short code.
- **edit_document** — Search-and-replace edit. Validates frontmatter integrity.
- **list_documents** — List all documents with type/phase/parent.
- **search_documents** — Text search with type filter and limit.
- **transition_phase** — Move document to next phase. Use `force: true` to skip exit criteria.
- **archive_document** — Archive document and all children.
- **reassign_parent** — Move a task to a different initiative.

## Quality Tools

- **capture_quality_baseline** — Parse eslint/clippy/tsc/coverage output into an AnalysisBaseline.
- **compare_quality_baselines** — Compare two baselines, create a QualityRecord.
- **list_quality_records** — List quality records and baselines with status filter.
- **check_architecture_conformance** — Validate files against a ReferenceArchitecture.

## Rule Tools

- **query_rules** — Filter rules by scope/protection/architecture ref.
- **get_applicable_rules** — Scope-inheritance-aware query (Platform > Org > Repo > Package > Component > Task).
- **list_protected_rules** — Governance audit: all protected rules.

## Insight Note Tools

- **create_insight_note** — Record reusable knowledge with scope and category.
- **fetch_insight_notes** — Get notes matching repo/package/subsystem/path/symbol scope. Increments fetch count.
- **score_insight_note** — Record helpful/meh/harmful feedback. Auto-detects prune candidates.
- **list_insight_notes** — List notes with status/category filters and stats.

## Traceability Tools

- **create_cross_reference** — Link documents with typed relationships (governs, references, validates, blocks, etc.).
- **query_relationships** — Find outgoing/incoming/all relationships for a document.
- **trace_ancestry** — Walk ancestors, descendants, or siblings via parent_child relationships.
- **list_cross_references** — List all cross-references with type/involvement filters.

## Architecture Tools

- **query_architecture_catalog** — Search catalog by language/project type.
- **list_catalog_languages** — Browse available languages and project types.
- **read_reference_architecture** — Read the project's selected architecture.
- **evaluate_brownfield** — Score how well the repo matches a catalog entry.

## Code Tools

- **index_code** — Index source symbols with tree-sitter. Query by name/kind.
- **initialize_project** — Initialize a new .cadre directory.

## Workflow Recipes

### Start a task with context
1. `fetch_insight_notes` with scope matching your work area
2. `get_applicable_rules` at the relevant scope level
3. `read_document` on the parent initiative/story

### Record a discovery
1. `create_insight_note` with appropriate scope and category
2. `create_cross_reference` linking the note to related documents

### Quality check before phase transition
1. `capture_quality_baseline` with fresh tool output
2. `compare_quality_baselines` against the previous baseline
3. `check_architecture_conformance` if architecture rules exist

### Evaluate architecture for a new repo
1. `list_catalog_languages` to see available patterns
2. `evaluate_brownfield` with the detected language/project type
3. Create a `reference_architecture` document based on results

## Scope Inheritance (Rules)

Rules at broader scopes apply to narrower scopes:
Platform > Organization > Repository > Package > Component > Task

## Insight Note Categories

hotspot_warning, recurring_failure, misleading_name, validation_hint,
local_exception, boundary_warning, subsystem_quirk

## Relationship Types

parent_child, governs, references, derived_from, supersedes,
conflicts_with, validates, blocks, approved_by
";

/// The prompt name used in MCP prompts/list and prompts/get.
pub const PROMPT_NAME: &str = "cadre-guide";

/// The prompt description.
pub const PROMPT_DESCRIPTION: &str =
    "Complete tool reference, domain concepts, and workflow recipes for Cadre";
