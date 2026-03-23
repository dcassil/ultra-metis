---
id: ci-workflow-test-clippy-and-format
level: task
title: "CI Workflow: Test, Clippy, and Format Checks on PR/Push"
short_code: "SMET-T-0139"
created_at: 2026-03-20T17:45:28.733442+00:00
updated_at: 2026-03-20T19:41:14.929836+00:00
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

# CI Workflow: Test, Clippy, and Format Checks on PR/Push

## Parent Initiative

[[SMET-I-0050]]

## Objective

Create a GitHub Actions CI workflow (`.github/workflows/ci.yml`) that runs on every push to `main` and on every pull request targeting `main`. This workflow ensures code quality by running the full test suite, Clippy linting, and format checking before code can be merged.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.github/workflows/ci.yml` exists and triggers on push to main + PRs targeting main
- [ ] Workflow runs `cargo test --workspace` and fails the build on any test failure
- [ ] Workflow runs `cargo clippy --workspace --all-targets -- -D warnings` and fails on any warnings
- [ ] Workflow runs `cargo fmt --all -- --check` and fails if any files are unformatted
- [ ] Rust caching via `Swatinem/rust-cache@v2` is configured to speed up repeat builds
- [ ] Workflow installs the stable Rust toolchain via `dtolnay/rust-toolchain@stable`
- [ ] Workflow runs on `ubuntu-latest` (cheapest runner, sufficient for checks)
- [ ] Pipeline completes in under 10 minutes for a clean cache run

## Implementation Notes

### File to Create
`.github/workflows/ci.yml`

### Technical Approach

Create a workflow with three parallel jobs (test, clippy, fmt) that run independently so failures are reported per-check rather than stopping at the first failure.

**Workflow structure:**
- **Trigger**: `on: push: branches: [main]` and `on: pull_request: branches: [main]`
- **Job 1 - Test Suite**: `cargo test --workspace` — runs all unit and integration tests across all crates
- **Job 2 - Clippy Lint**: `cargo clippy --workspace --all-targets -- -D warnings` — treats all clippy warnings as errors for strict quality
- **Job 3 - Format Check**: `cargo fmt --all -- --check` — verifies all Rust code is properly formatted

Each job follows the same setup pattern:
1. `actions/checkout@v4` to get the code
2. `dtolnay/rust-toolchain@stable` to install Rust (with `clippy` component for the lint job, `rustfmt` component for the fmt job)
3. `Swatinem/rust-cache@v2` to cache compiled dependencies (keyed by `Cargo.lock` hash)
4. Run the actual check command

### Key Decisions
- Use `-D warnings` for clippy to enforce zero-warning policy from the start
- Run on `ubuntu-latest` since these are code quality checks, not platform-specific builds
- Parallel jobs over sequential steps for faster feedback and independent failure reporting
- Cache is keyed by `Cargo.lock` hash automatically by `rust-cache`

### Dependencies
None — this task is fully independent and can be done first.

### Verification
- Push a branch with a deliberate clippy warning and verify CI fails on the lint job
- Push a branch with an unformatted file and verify CI fails on the fmt job
- Push a branch with a failing test and verify CI fails on the test job
- Push a clean branch and verify all three jobs pass green

### Estimated Effort
1-2 days

## Status Updates

- **2026-03-20**: Created `.github/workflows/ci.yml` with three parallel jobs (test, clippy, fmt). Each job uses `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, and `Swatinem/rust-cache@v2`. Test runs `cargo test --workspace`, clippy runs with `-D warnings`, fmt runs with `--check`. All trigger on push to main and PRs targeting main. All acceptance criteria met.