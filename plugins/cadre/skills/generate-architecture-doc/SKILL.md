---
name: cadre:generate-architecture-doc
description: "Generate an architecture catalog document from industry best practices for a given language and project type. Use when the user asks to 'generate architecture doc', 'create catalog entry', 'new architecture pattern', 'add architecture to catalog', or wants to create a reusable architecture reference for a language/framework combination."
---

# Generate Architecture Doc from Best Practices

You are generating a new architecture catalog entry for the Cadre architecture catalog. This entry will describe a well-known software architecture pattern that Cadre uses for greenfield project scaffolding, brownfield evaluation, and rules seeding.

## Input

Parse the user's request for:
- **Language** (required): e.g., "Go", "Java", "TypeScript", "Ruby", "C#"
- **Project type/function** (required): e.g., "microservice", "monolith", "CLI tool", "GraphQL API", "mobile app"

If either is missing, ask the user before proceeding.

## Step 1: Check for Duplicates

Use the `query_architecture_catalog` MCP tool to check if a similar entry already exists:
- Query by language
- Check if the project type or a close variant is already in the catalog

If a match exists, inform the user:
> "An architecture doc for **{language} / {project_type}** already exists in the catalog. Would you like to create a variant, or skip?"

If the user wants to proceed anyway, continue. Otherwise, stop.

## Step 2: Research and Generate

Based on your knowledge of industry best practices for the given language + project type combination, generate a complete architecture catalog entry.

### YAML Frontmatter Schema

The entry MUST have this exact YAML frontmatter structure:

```yaml
---
id: CADRE-AC-{LANG}-{TYPE}
level: architecture_catalog_entry
title: "{Language} {Project Type} ({Framework/Tool})"
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
  - "src/"
  - ...
layers:
  - "layer1"
  - ...
module_boundaries:
  - "boundary1"
  - ...
dependency_rules:
  - "layer1 -> layer2"
  - ...
naming_conventions:
  - "convention1"
  - ...
anti_patterns:
  - "anti-pattern1"
  - ...
rules_seed_hints:
  - "rule-hint-1"
  - ...
analysis_expectations:
  - "expectation-1"
  - ...
---
```

**Field guidelines:**
- `id` and `short_code`: Use format `CADRE-AC-{LANG_ABBREV}-{TYPE_ABBREV}` (e.g., `CADRE-AC-GO-MICRO`, `CADRE-AC-JAVA-REST`)
- `language`: lowercase (e.g., "go", "java", "ruby")
- `project_type`: kebab-case (e.g., "microservice", "cli-tool", "graphql-api")
- `folder_layout`: At least 8-10 entries showing the expected directory tree with `src/`, test dirs, config dirs
- `layers`: The 3-5 primary architectural layers (e.g., "handlers", "services", "repositories")
- `module_boundaries`: How code is organized into modules/packages
- `dependency_rules`: At least 4 rules describing allowed dependencies between layers (use "->")
- `naming_conventions`: At least 5 naming patterns for files, functions, types
- `anti_patterns`: At least 4 common mistakes specific to this architecture
- `rules_seed_hints`: Rules that Cadre's analysis engine should generate (format: "rule-name: description")
- `analysis_expectations`: Quality tool expectations (e.g., "lint-clean", "type-safe", "minimum-test-coverage-80")

### Markdown Body

After the frontmatter, include these sections:

```markdown
# {Title}

## Overview
{2-3 sentences describing the architecture pattern, when to use it, and what it solves}

## Structure
{Describe the concrete organization: what goes where, how layers interact}

## Dependency Rules
{Bullet list of dependency constraints}

## Anti-Patterns
{Bullet list of common mistakes to avoid}

## Quality Expectations
{Bullet list of quality/tooling expectations}
```

## Step 3: Save Locally

Use the Write tool to save the generated doc to the path the user specifies. If no path is specified, save to the current directory as `{language}-{project-type}-architecture.md`.

Show the user a summary of what was generated:
- Language / Project Type
- Number of layers, rules, conventions defined
- File path where it was saved

## Step 4: Optional PR Submission

Ask the user: "Would you like to submit this as a PR to the public catalog repo (`dcassil/cadre-architecture-docs`)?"

If yes, execute these steps:

```bash
# Clone or update the repo
gh repo clone dcassil/cadre-architecture-docs /tmp/cadre-arch-docs 2>/dev/null || git -C /tmp/cadre-arch-docs pull --ff-only

# Create language directory if needed
mkdir -p /tmp/cadre-arch-docs/{language}

# Copy the generated doc
cp {saved_file_path} /tmp/cadre-arch-docs/{language}/{project-type}.md

# Create branch, commit, push
git -C /tmp/cadre-arch-docs checkout -b add-{language}-{project-type}
git -C /tmp/cadre-arch-docs add .
git -C /tmp/cadre-arch-docs commit -m "Add {language} {project-type} architecture doc"
git -C /tmp/cadre-arch-docs push -u origin add-{language}-{project-type}

# Create PR
gh pr create --repo dcassil/cadre-architecture-docs \
  --title "Add {language} {project-type} architecture doc" \
  --body "## New Architecture Entry

- **Language**: {language}
- **Project Type**: {project-type}
- **Layers**: {comma-separated layers}
- **Source**: Generated from industry best practices by Cadre

### What this adds
A new architecture catalog entry for {language} {project-type} projects, including folder layout, dependency rules, naming conventions, anti-patterns, and quality expectations.

Generated with [Cadre](https://github.com/dcassil/cadre)"
```

Report the PR URL to the user when done.

If the user declines PR submission, just confirm the local file is saved and they can submit manually later.
