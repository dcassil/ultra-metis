---
id: brownfield-architecture-evaluation
level: initiative
title: "Brownfield Architecture Evaluation"
short_code: "SMET-I-0028"
created_at: 2026-03-11T21:52:32.607504+00:00
updated_at: 2026-03-17T01:33:25.206072+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"
  - "#feature-architecture"
  - "#category-architecture"


exit_criteria_met: false
estimated_complexity: M
strategy_id: cadre-core-engine-repo
initiative_id: brownfield-architecture-evaluation
---

# Brownfield Architecture Evaluation

## Context

Most real repos are brownfield — they have existing code with an existing (often implicit) architecture. When CADRE initializes in a brownfield repo, it needs to analyze the current structure, assess its quality, and either match it to a catalog pattern or derive a stable reference from it.

This is the analysis and matching counterpart to the greenfield selection flow (SMET-I-0027). The goal: every brownfield repo ends up with a persisted Reference Architecture, whether matched, derived, or recommended.

Split from the original SMET-I-0016 (now archived).

## Goals & Non-Goals

**Goals:**
- Build structure analyzer: detect folder organization, file patterns, directory naming conventions
- Build import graph analyzer: detect layering, dependency direction, module boundaries, circular dependencies
- Build naming convention detector: file naming, export patterns, test file placement
- Build architecture quality scorer: produce a quality score with specific findings (circular deps, layering violations, inconsistent patterns, dead code clustering, test coverage gaps)
- **Good architecture path**: match existing structure against catalog patterns (from SMET-I-0027), confirm match, create Reference Architecture linking to catalog entry
- **Good architecture path (no match)**: capture existing structure as a derived Reference Architecture
- **Bad architecture path**: recommend a catalog pattern, explain problems and refactor scope, present accept/decline choice to user
- **Declined path**: record current architecture as-is with quality findings noted
- User always has final say — system recommends, does not force

**Non-Goals:**
- The catalog entries themselves — covered by SMET-I-0027
- Domain types — covered by SMET-I-0020
- Forcing architecture rewrites — user always decides
- Deep framework-specific analysis (e.g., understanding Next.js routing)
- Non-JS/TS analysis — start with JS/TS, expand later

## Detailed Design

### Structure Analyzer
- Walk directory tree, build folder structure model
- Detect common patterns: `src/`, `lib/`, `test/`, `__tests__/`, feature-based folders, layer-based folders
- Identify package boundaries (package.json locations in monorepos)

### Import Graph Analyzer
- Parse import/require statements to build dependency graph
- Detect: layering patterns, dependency direction, circular dependencies, cross-boundary imports
- Produce: layer model (which directories act as layers), boundary model (what's public vs internal)

### Naming Convention Detector
- Analyze file names: camelCase, kebab-case, PascalCase patterns
- Analyze export patterns: default vs named, barrel files
- Analyze test patterns: co-located vs separate, naming conventions (`.test.ts`, `.spec.ts`)

### Quality Scorer
- Inputs: structure analysis, import graph, naming conventions, static analysis results (if available)
- Scoring criteria: circular dependency count, layering violation count, naming consistency score, test coverage distribution, dead code indicators
- Output: overall quality score (0-100) with itemized findings and severity levels
- Threshold: configurable, default ~70 for "good" vs "bad" classification

### Good Architecture Path
- Score existing structure against each matching catalog entry
- Best match above threshold: create Reference Architecture linked to that catalog entry, note deviations
- No catalog match but quality score is good: create derived Reference Architecture documenting actual patterns

### Bad Architecture Path
- Identify project type/intent from existing code
- Recommend specific catalog pattern that would be a strong fit
- Present to user: "Your current architecture has these problems [findings]. The recommended pattern would address them. Adopting it would require refactoring [scope estimate]."
- Accept: catalog pattern becomes Reference Architecture, system creates refactoring initiative
- Decline: current structure recorded as-is with findings preserved

## Chosen Design

**Approach**: New `brownfield_evaluator` module under `domain/catalog/` with three core components:

1. **StructureAnalyzer** - Walks file paths, detects folder patterns, identifies layers and naming conventions
2. **PatternMatcher** - Scores existing structure against catalog entries using folder layout overlap, layer detection, and naming convention matching
3. **BrownfieldEvaluator** - Orchestrator that combines analysis + matching to produce an `EvaluationResult` with one of four outcomes:
   - `CatalogMatch` (good arch, matches catalog) -> creates linked ReferenceArchitecture
   - `DerivedArchitecture` (good arch, no match) -> creates derived ReferenceArchitecture
   - `RecommendCatalogPattern` (bad arch) -> recommends pattern with findings
   - `RecordAsIs` (declined recommendation) -> records current state

**Quality scoring**: Simple heuristic based on naming consistency, layer presence, and folder structure regularity. Score 0-100, threshold at 70.

**Design decisions**:
- Pure functions operating on file path lists (no filesystem I/O in domain)
- Reuses existing `ParsedToolOutput`, `FindingEntry`, `MetricEntry` from quality/types
- Reuses `CatalogQueryEngine` for catalog matching
- Reuses `ReferenceArchitecture` for output

## Alternatives Considered

1. **Skip brownfield, only support greenfield**: Rejected — most repos are brownfield. This is essential.
2. **Always force a catalog match**: Rejected — some repos have valid custom architectures that don't match any catalog entry.
3. **Pure AI-based evaluation**: Rejected — heuristic analysis is more predictable and explainable. AI can assist but shouldn't be the sole evaluator.

## Implementation Plan

Phase 1: Build structure analyzer
Phase 2: Build import graph analyzer
Phase 3: Build naming convention detector
Phase 4: Build quality scorer
Phase 5: Build catalog pattern matching/scoring
Phase 6: Build good-architecture path (match or derive)
Phase 7: Build bad-architecture path (recommend, explain, user choice)
Phase 8: Build declined-bad fallback (record as-is)
Phase 9: Integration test with real brownfield repos

## Acceptance Criteria

- Structure analysis correctly identifies folder patterns and conventions
- Import graph analysis detects layering, boundaries, and circular dependencies
- Quality scorer produces consistent, explainable scores with itemized findings
- Good architecture is correctly matched to catalog entries or captured as derived
- Bad architecture triggers recommendation with clear explanation and refactor scope
- User always has the final say (accept recommendation or keep current)
- Every brownfield repo ends up with a persisted Reference Architecture

## Risks / Dependencies

- Depends on SMET-I-0020 for Reference Architecture type
- Depends on SMET-I-0027 for catalog entries to match against
- Heuristic analysis is inherently imperfect — must be transparent about confidence
- Scoring thresholds may need tuning based on real-world repos
- Must coordinate with SMET-I-0008 (bootstrap flow triggers evaluation)