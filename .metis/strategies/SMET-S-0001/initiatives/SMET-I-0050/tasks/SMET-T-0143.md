---
id: extend-makefile-with-ci-release
level: task
title: "Extend Makefile with CI, Release-Local, and Package Targets"
short_code: "SMET-T-0143"
created_at: 2026-03-20T17:45:32.877105+00:00
updated_at: 2026-03-20T19:43:21.072325+00:00
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

# Extend Makefile with CI, Release-Local, and Package Targets

## Parent Initiative

[[SMET-I-0050]]

## Objective

Extend the existing `Makefile` with additional targets so developers can run the full CI suite locally, build release binaries for their current platform, and create distributable packages without needing to push to GitHub. This ensures parity between local development and CI, and provides a fast feedback loop.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `make ci` runs the full CI suite locally: test + clippy + format check (matching the GitHub Actions CI workflow)
- [ ] `make lint` runs `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `make fmt` runs `cargo fmt --all` to auto-format code
- [ ] `make fmt-check` runs `cargo fmt --all -- --check` (matches CI behavior, check-only)
- [ ] `make release-local` builds both binaries in release mode for the current platform
- [ ] `make package` runs `scripts/package.sh` with a local version tag to create a distributable archive
- [ ] All existing targets (`build`, `build-mcp`, `build-cli`, `install`, `install-binary`, `test`, `clean`) continue to work unchanged
- [ ] `.PHONY` declarations are updated for all new targets
- [ ] Running `make ci` on a clean, correctly formatted checkout passes all checks

## Implementation Notes

### File to Modify
`Makefile` (at repo root — already exists with `build`, `install`, `test`, `clean` targets)

### Technical Approach

Add the following new targets to the existing Makefile:

- **`ci`**: Depends on `test`, `lint`, `fmt-check`. Runs them sequentially (if any fails, stops). Matches the GitHub Actions CI workflow so developers can catch issues before pushing.
- **`lint`**: Runs clippy with `-D warnings` (same flags as CI). Useful during development for quick lint checks.
- **`fmt`**: Runs `cargo fmt --all` to auto-format all Rust code. This is the developer convenience target (modifies files).
- **`fmt-check`**: Runs `cargo fmt --all -- --check` (read-only, fails if anything is unformatted). This is what CI runs.
- **`release-local`**: Depends on the existing `build` target. Prints the binary paths and sizes for confirmation.
- **`package`**: Depends on `release-local`. Extracts the version from `Cargo.toml` workspace, appends `-local` suffix, and calls `scripts/package.sh`. Creates a distributable archive in `dist/`.

### Key Design Decisions
- **`ci` runs sequentially**: Unlike GitHub Actions where test/clippy/fmt run in parallel, locally they run sequentially so that a failing test stops before waiting for clippy. This is simpler and faster for iterative dev.
- **`release-local` reuses `build`**: The existing `build` target already does `cargo build --release` for both binaries.
- **`package` extracts version from Cargo.toml**: Uses `grep` + `sed` to pull the workspace version, appends `-local` to distinguish local packages from CI releases.
- **`fmt` vs `fmt-check` split**: `fmt` modifies files (developer use), `fmt-check` is read-only (CI verification).

### Dependencies
- SMET-T-0142 (packaging script) must exist for `make package` to work
- All other targets are self-contained

### Verification
- Run `make ci` on a clean checkout and verify all checks pass
- Introduce a clippy warning and verify `make lint` catches it
- Run `make fmt` on unformatted code and verify it formats it
- Run `make release-local` and verify binaries exist at expected paths
- Run `make package` (after packaging script exists) and verify archive is created in `dist/`

### Estimated Effort
1 day

## Status Updates

- **2026-03-20**: Extended Makefile with all new targets: `ci` (runs test + lint + fmt-check sequentially), `lint` (clippy with -D warnings), `fmt` (auto-format), `fmt-check` (check-only), `release-local` (builds + shows binary paths/sizes), `package` (extracts version from Cargo.toml, appends -local suffix, calls scripts/package.sh). Updated `clean` to also remove `dist/`. All `.PHONY` declarations updated. All existing targets preserved unchanged. All acceptance criteria met.