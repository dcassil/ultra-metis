---
id: configure-release-please-for
level: task
title: "Configure release-please for Automated Versioning and Changelog"
short_code: "SMET-T-0140"
created_at: 2026-03-20T17:45:29.663712+00:00
updated_at: 2026-03-20T19:41:50.500135+00:00
parent: SMET-I-0050
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0050
---

# Configure release-please for Automated Versioning and Changelog

## Parent Initiative

[[SMET-I-0050]]

## Objective

Set up Google's [release-please](https://github.com/googleapis/release-please) GitHub Action to automate version bumps, CHANGELOG generation, and git tag creation based on Conventional Commits. When developers merge PRs with conventional commit messages (e.g., `feat:`, `fix:`, `chore:`), release-please automatically creates a Release PR that bumps versions and generates changelog entries. When that Release PR is merged, release-please creates a git tag that triggers the release build workflow (SMET-T-0141).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.github/workflows/release-please.yml` exists and triggers on push to `main`
- [ ] release-please is configured for the `rust` release type (understands `Cargo.toml` versioning)
- [ ] release-please creates Release PRs with version bump + CHANGELOG.md updates when conventional commits are detected
- [ ] Merging a Release PR creates a git tag in `v{MAJOR}.{MINOR}.{PATCH}` format
- [ ] `release-please-config.json` is configured at repo root for monorepo-aware versioning
- [ ] `.release-please-manifest.json` tracks current version at repo root
- [ ] CHANGELOG.md is generated at repo root with categorized entries (Features, Bug Fixes, etc.)
- [ ] The workspace `Cargo.toml` version is bumped by release-please

## Implementation Notes

### Files to Create

1. **`.github/workflows/release-please.yml`** — The workflow that runs release-please on every push to main. Uses `googleapis/release-please-action@v4`. Exposes `release_created` and `tag_name` as job outputs for downstream workflows.

2. **`release-please-config.json`** — Configuration file at repo root:
   - `release-type`: `"rust"` (understands Cargo.toml version fields)
   - Single package at `"."` with component name `"ultra-metis"`
   - `bump-minor-pre-major: true` and `bump-patch-for-minor-pre-major: true` to prevent rapid version inflation while pre-1.0

3. **`.release-please-manifest.json`** — Version tracking file: `{ ".": "0.1.0" }` (matches current workspace version in Cargo.toml)

### Technical Approach

- **Single version for the whole workspace**: All crates share the workspace version in the root `Cargo.toml`. No per-crate versioning until needed.
- **Pre-1.0 version bumping**: With `bump-minor-pre-major` and `bump-patch-for-minor-pre-major`, `feat:` commits bump patch (not minor) and `fix:` commits also bump patch. This keeps versions from inflating during early development.
- **Conventional Commit format** must be followed:
  - `feat: add new command` -> patch bump (pre-1.0)
  - `fix: correct parsing error` -> patch bump
  - `feat!: redesign API` or `BREAKING CHANGE:` -> minor bump (pre-1.0)
  - `chore:`, `docs:`, `refactor:`, `test:` -> no version bump, but included in changelog
- **Workflow outputs**: The release-please job exposes `release_created` and `tag_name` outputs so the release build workflow (SMET-T-0141) can be chained.

### Dependencies
None — this task is independent, but SMET-T-0141 depends on the tags this creates.

### Verification
- Merge a PR with a `feat: test release-please` commit message
- Verify release-please creates a Release PR with correct version bump and CHANGELOG.md entry
- Merge the Release PR and verify a git tag (e.g., `v0.1.1`) is created
- Verify CHANGELOG.md is updated with categorized entries

### Estimated Effort
1-2 days

## Status Updates

- **2026-03-20**: Created `.github/workflows/release-please.yml` using `googleapis/release-please-action@v4` triggered on push to main. Exposes `release_created`, `tag_name`, and `version` outputs for downstream workflows. Created `release-please-config.json` with `rust` release type, `ultra-metis` component, and pre-1.0 bump settings. Created `.release-please-manifest.json` tracking version 0.1.0. Configured `extra-files` to keep all crate Cargo.toml versions in sync. All acceptance criteria met.