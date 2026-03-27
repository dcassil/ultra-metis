---
name: cadre:contribute-brownfield-architecture
description: "Evaluate a brownfield codebase and contribute its architecture as a catalog entry if it's high-quality and unique. Use when the user asks to 'contribute architecture', 'submit architecture to catalog', 'brownfield contribute', 'share this architecture', 'add this project's architecture to the catalog', or wants to contribute a real-world architecture pattern from an existing codebase."
---

# Evaluate Brownfield and Contribute Architecture Doc

You are evaluating an existing codebase's architecture and, if it's high-quality and unique, generating a catalog entry and submitting it as a PR to the public Cadre architecture catalog.

## Input

Parse the user's request for:
- **Project path** (optional): path to the codebase to evaluate. Defaults to the current project directory.
- **Language** (required for evaluate_brownfield): the primary language of the project
- **Project type** (required for evaluate_brownfield): the type of project (e.g., "server", "cli-tool", "web-app")

If language or project type are missing, ask the user.

## Step 1: Evaluate the Codebase

Use the `evaluate_brownfield` MCP tool:
- `project_path`: the path to the project
- `language`: the language
- `project_type`: the project type

Read the evaluation result carefully. Extract:
- **Quality Score**: the percentage quality score
- **Outcome**: CatalogMatch, DerivedArchitecture, RecommendCatalogPattern, or RecordAsIs
- **Files Analyzed**: count of files evaluated

Report the evaluation summary to the user:
> **Brownfield Evaluation Results**
> - Quality Score: {score}%
> - Outcome: {outcome}
> - Files Analyzed: {count}

## Step 2: Quality Gate

**If quality score < 70%**, stop and report:
> This codebase scored **{score}%** on architecture quality, which is below the 70% threshold for catalog contribution. Common issues found: {list issues from findings}.
>
> To improve the architecture before contributing, consider: {suggestions based on findings}.

Do NOT proceed to submission.

**If quality score >= 70%**, continue to Step 3.

## Step 3: Uniqueness Check

Use `query_architecture_catalog` to check for existing entries with the same language:
- Query by language to get all entries for that language

Compare the evaluated architecture against existing entries:

1. **Exact project type match**: If an entry with the same `language` + `project_type` exists, compare layers:
   - Count how many detected layers overlap with the existing entry's layers
   - If overlap > 80%, report: "A very similar entry already exists: **{existing title}**. The detected architecture shares {overlap}% of layers."
   - Ask the user if they want to proceed anyway (creating a variant) or stop

2. **No exact match**: The architecture is unique. Proceed to Step 4.

3. **Partial match**: If the overlap is 40-80%, inform the user: "A related entry exists (**{existing title}**), but your architecture has meaningful differences. Proceeding with contribution."

## Step 4: Generate Architecture Doc

Based on the brownfield evaluation results and your analysis of the codebase, generate a complete architecture catalog entry.

### Mapping Evaluation to Schema

Use the evaluation output to populate the YAML frontmatter:

```yaml
---
id: CADRE-AC-{LANG}-{TYPE}
level: architecture_catalog_entry
title: "{Language} {Project Type} ({detected framework/pattern})"
short_code: "CADRE-AC-{LANG}-{TYPE}"
created_at: {current ISO 8601 timestamp}
updated_at: {current ISO 8601 timestamp}
archived: false

tags:
  - "#architecture_catalog_entry"
  - "#phase/published"

exit_criteria_met: false
schema_version: 1
epic_id: NULL

language: "{language}"
project_type: "{project-type}"
folder_layout:
  # From the evaluated directory structure
  - "{detected folders}"
layers:
  # From the detected architectural layers
  - "{detected layers}"
module_boundaries:
  # From detected module boundaries
  - "{detected boundaries}"
dependency_rules:
  # Inferred from layer relationships in the evaluation
  - "{layer1} -> {layer2}"
naming_conventions:
  # From detected naming convention (KebabCase, PascalCase, SnakeCase, CamelCase)
  - "{detected naming patterns}"
anti_patterns:
  # Common anti-patterns for this architecture style
  - "{anti-patterns relevant to the detected pattern}"
rules_seed_hints:
  # Generated from the analysis findings
  - "{rule hints based on evaluation}"
analysis_expectations:
  # From quality findings
  - "{quality expectations based on evaluation}"
---
```

### Markdown Body

```markdown
# {Title}

## Overview
{Describe the architecture based on what was detected: layers, structure, purpose}

## Structure
{Describe the folder organization and how layers interact, based on evaluation}

## Dependency Rules
{List dependency constraints inferred from the layer structure}

## Anti-Patterns
{List anti-patterns relevant to this architecture style}

## Quality Expectations
{List quality expectations based on the evaluation findings}
```

### Important Guidelines
- Base the doc on what was ACTUALLY detected, not hypothetical best practices
- The value of brownfield contributions is that they represent REAL working architectures
- Include specific folder paths and naming patterns from the actual codebase
- Anonymize any project-specific names (company names, product names) in the doc

## Step 5: Save and Review

Save the generated doc locally using the Write tool:
- Default path: `./{language}-{project-type}-architecture.md` in the current directory

Show the user the generated doc summary and ask for confirmation:
> **Generated Architecture Doc**
> - Title: {title}
> - Language: {language} / Project Type: {project-type}
> - Layers: {comma-separated layers}
> - Quality Score: {score}%
> - Uniqueness: {unique / variant of existing}
>
> The doc has been saved to `{path}`. Would you like to submit it as a PR to the catalog?

## Step 6: PR Submission

If the user confirms, submit the doc as a PR:

```bash
# Clone or update the repo
gh repo clone dcassil/cadre-architecture-docs /tmp/cadre-arch-docs 2>/dev/null || git -C /tmp/cadre-arch-docs pull --ff-only

# Ensure we're on main and up to date
git -C /tmp/cadre-arch-docs checkout main 2>/dev/null
git -C /tmp/cadre-arch-docs pull --ff-only 2>/dev/null

# Create language directory if needed
mkdir -p /tmp/cadre-arch-docs/{language}

# Copy the generated doc
cp {saved_file_path} /tmp/cadre-arch-docs/{language}/{project-type}.md

# Create branch, commit, push
git -C /tmp/cadre-arch-docs checkout -b contribute-{language}-{project-type}
git -C /tmp/cadre-arch-docs add .
git -C /tmp/cadre-arch-docs commit -m "Contribute {language} {project-type} architecture doc from brownfield evaluation"
git -C /tmp/cadre-arch-docs push -u origin contribute-{language}-{project-type}

# Create PR
gh pr create --repo dcassil/cadre-architecture-docs \
  --title "Contribute {language} {project-type} architecture doc" \
  --body "## Brownfield Architecture Contribution

### Evaluation Summary
| Metric | Value |
|--------|-------|
| Quality Score | {score}% |
| Files Analyzed | {count} |
| Detected Layers | {layers} |
| Naming Convention | {convention} |
| Uniqueness | {unique/variant} |

### What this adds
A new architecture catalog entry for **{language} {project-type}** projects, derived from evaluating a real-world codebase. This entry captures the actual architectural patterns, folder structure, and conventions used in production.

### Architecture Highlights
- **Layers**: {comma-separated layers}
- **Key dependency rules**: {top 2-3 rules}
- **Quality characteristics**: {top expectations}

---
Contributed via [Cadre](https://github.com/dcassil/cadre) brownfield evaluation"
```

Report the PR URL to the user.

If the user declines, confirm the local file is saved and stop.
