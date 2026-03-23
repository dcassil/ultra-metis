---
id: build-release-and-distribution
level: initiative
title: "Build, Release, and Distribution Strategy for Monorepo Artifacts"
short_code: "SMET-I-0050"
created_at: 2026-03-17T21:38:34.444414+00:00
updated_at: 2026-03-23T20:41:45.248012+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: build-release-and-distribution
---

# Build, Release, and Distribution Strategy for Monorepo Artifacts Initiative

## Context

Cadre is now structured as a monorepo with multiple distinct components that serve very different deployment targets and user installation workflows:

| Component | Location | What it is | Where it goes |
|-----------|----------|------------|---------------|
| **cadre-mcp** | `crates/cadre-mcp/` | MCP server binary | Needs to be runnable by Claude Code via `.mcp.json` |
| **cadre-cli** | `crates/cadre-cli/` | CLI tool | Installed on user's PATH for terminal use |
| **cadre-core** | `crates/cadre-core/` | Shared Rust library | Compiled into binaries (not distributed standalone) |
| **cadre-store** | `crates/cadre-store/` | Persistence layer | Compiled into binaries (not distributed standalone) |
| **plugin.json** | `plugin.json` | Claude Code plugin manifest | Must be in a directory Claude Code can reference as a plugin |
| **control-web** | `apps/control-web/` (future) | Next.js dashboard | Deployed to a web server (Cloudflare, Vercel, etc.) |
| **control-api** | `apps/control-api/` (future) | API gateway | Deployed as a server |
| **machine-runner** | `apps/machine-runner/` (future) | Local daemon | Installed alongside CLI on user's machine |

The problem: there's no defined strategy for how release builds are structured, where built artifacts land, how users install or reference them, or what tooling manages the build/release pipeline. Without this, every new component we add creates ad-hoc decisions about distribution.

## Discovery Findings (2026-03-18)

The immediate plugin installation gap was identified during hands-on testing. The current `plugin.json` uses `cargo run` to launch the MCP server, which:
- Requires the Rust toolchain on the user's machine
- Has slow startup (recompilation checks on every launch)
- Only works when referenced from the repo directory

A separate initiative ([[SMET-I-0058]]) has been created to handle the immediate plugin installability work. This initiative (SMET-I-0050) retains the broader build, release, and cross-platform distribution concerns.

## Goals & Non-Goals

**Goals:**
- CI/CD pipeline for automated cross-platform release builds (GitHub Actions)
- GitHub Releases with pre-built binaries for macOS (x86_64, aarch64), Linux (x86_64, aarch64), Windows (x64)
- Homebrew tap for macOS CLI distribution
- npm package wrapper for `npx cadre` style usage (future)
- Plugin marketplace / registry publishing (when available)
- Monorepo build orchestration tooling (just, cargo-make, or similar)
- CLI distribution strategy (`cargo install`, homebrew, GitHub releases)
- Future web/server component deployment strategy (Docker, Cloudflare, etc.)
- Release versioning strategy (release-please, conventional commits, etc.)

**Non-Goals:**
- Immediate local plugin installation (handled by separate initiative)
- Actually building the web app, API, or daemon (those are separate initiatives)
- Package registry publishing to crates.io (post-v1)

## Key Questions to Answer in Discovery

1. **Plugin distribution**: How do users install the Claude Code plugin? Git clone? Plugin registry? Cargo install? Pre-built binary download?
2. **MCP binary location**: Should `.mcp.json` point to a `cargo run` invocation (dev mode) or a pre-built binary (release mode)? Where does the binary live?
3. **CLI installation**: `cargo install`? Homebrew? Download from GitHub releases?
4. **Monorepo build tool**: Do we need `just`, `make`, `cargo-make`, or similar to orchestrate cross-component builds?
5. **Release artifact structure**: What does a "release" look like? A GitHub release with binaries? A plugin directory with everything bundled?
6. **Future web/server components**: How do `apps/control-web` and `apps/control-api` get built and deployed? Docker? Direct deploy?

## Detailed Design

### Design Decisions

**1. CI/CD Platform: GitHub Actions**
GitHub Actions is the natural choice — the repo is on GitHub, the original Metis project already used Actions for CI and release, and it provides free cross-platform runners (macOS, Linux, Windows). No external CI service needed.

**2. Release Versioning: Conventional Commits + release-please**
Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages and Google's [release-please](https://github.com/googleapis/release-please) GitHub Action for automated version bumps and changelog generation. This approach:
- Automatically determines semver bump from commit history (feat = minor, fix = patch, breaking = major)
- Creates release PRs with changelogs that are human-reviewable before merge
- Tags releases on merge, triggering the release build workflow
- Supports monorepo mode with per-crate versioning if needed later

**3. Cross-Platform Build Matrix**
Target the same platforms the original Metis supported, plus macOS x86_64:

| Target Triple | Runner | Notes |
|---------------|--------|-------|
| `aarch64-apple-darwin` | `macos-latest` | Apple Silicon (primary dev target) |
| `x86_64-apple-darwin` | `macos-latest` | Intel Macs (cross-compile from ARM runner) |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | Standard Linux |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04` | Linux ARM (cross-compile via `cross`) |
| `x86_64-pc-windows-msvc` | `windows-latest` | Windows |

Each release produces two binaries per platform: `cadre` (CLI) and `cadre-mcp` (MCP server).

**4. Release Artifact Packaging**
Each platform's artifacts are packaged as:
- Tarball (`.tar.gz`) for macOS/Linux: contains both binaries + LICENSE + README
- Zip (`.zip`) for Windows: contains both binaries + LICENSE + README
- SHA256 checksums file for verification
- Naming convention: `cadre-{version}-{target-triple}.tar.gz`

**5. GitHub Releases as Primary Distribution**
GitHub Releases serve as the canonical artifact store. All other distribution channels pull from here:
- Pre-built binaries attached to each release
- Release notes auto-generated from conventional commits
- Draft releases for review before publishing

**6. Homebrew Tap for macOS**
Create a separate `homebrew-cadre` tap repository with a formula that:
- Downloads the pre-built macOS binary from GitHub Releases
- Installs both `cadre` and `cadre-mcp` to the Homebrew prefix
- Auto-updated by a GitHub Action on release publish

**7. npm Wrapper Package (Future)**
An npm package (`cadre`) that:
- Downloads the appropriate platform binary on `postinstall`
- Provides `npx cadre` usage
- Enables Claude Code plugin installation via npm
- This is lower priority and can be a later task

**8. CI Pipeline Structure**
Two workflow files:
- **ci.yml**: Runs on every push/PR to main — `cargo test --workspace`, `cargo clippy`, `cargo fmt --check`
- **release.yml**: Triggered by release-please tags — cross-platform build matrix, artifact packaging, GitHub Release upload, Homebrew formula update

**9. Monorepo Build Orchestration**
Keep the existing Makefile as the primary developer build tool. Extend it with additional targets:
- `make ci` — run the full CI suite locally (test + clippy + fmt check)
- `make release-local` — build release binaries for the current platform
- `make package` — create distributable archive for current platform
- No need for `just` or `cargo-make` — Make is sufficient and already in use

**10. Rust Caching Strategy**
Use `Swatinem/rust-cache@v2` in all CI workflows to cache:
- `~/.cargo/registry` and `~/.cargo/git`
- `target/` directory
- Keyed by `Cargo.lock` hash and workflow name

### Architecture Overview

```
.github/
  workflows/
    ci.yml              # PR/push checks: test, clippy, fmt
    release.yml         # Tag-triggered: cross-platform build + release
scripts/
  package.sh            # Creates distributable archives from built binaries
Makefile                # Extended with ci, release-local, package targets
```

Release flow:
1. Developer merges PR with conventional commits
2. release-please creates a Release PR with version bump + changelog
3. Maintainer reviews and merges Release PR
4. release-please creates git tag (e.g., `v0.2.0`)
5. Tag triggers `release.yml` workflow
6. Workflow builds binaries for all platforms, packages them, uploads to GitHub Release
7. Separate job updates Homebrew tap formula with new version + checksums

## Alternatives Considered

**1. Cargo-only distribution (`cargo install cadre`)**
- Pros: Simple, standard Rust ecosystem, no binary hosting needed
- Cons: Requires Rust toolchain on user's machine, slow initial compile (5-10 min), can't distribute MCP server to non-Rust users
- Decision: Rejected as primary channel, but will support `cargo install` from crates.io post-v1

**2. Just / cargo-make instead of Make**
- Pros: `just` has better syntax, `cargo-make` integrates with Cargo ecosystem
- Cons: Additional dependency to install, Makefile already exists and works, team is familiar with Make
- Decision: Keep Make — it's already in use, universally available, and sufficient for our needs

**3. Manual GitHub Releases (no release-please)**
- Pros: Full control over version numbers and release timing
- Cons: Error-prone manual tagging, no automated changelog, easy to forget steps
- Decision: Use release-please for automation; maintainer still reviews Release PR before merge

**4. Single universal binary instead of separate CLI + MCP**
- Pros: Simpler distribution (one binary), subcommand-based routing
- Cons: Larger binary size for users who only need one, different execution models (CLI = interactive, MCP = stdio server), harder to configure in .mcp.json
- Decision: Keep separate binaries — they serve fundamentally different purposes

**5. Docker-based distribution**
- Pros: Zero-dependency installation, reproducible environment
- Cons: Overkill for CLI tools, adds Docker dependency, poor integration with Claude Code plugin system
- Decision: Rejected for CLI/MCP; will revisit for future server components (control-api)

**6. Cross-rs for cross-compilation**
- Pros: Handles cross-compilation toolchains automatically via Docker
- Cons: Adds complexity, slower builds, not needed for native-arch builds
- Decision: Use only for Linux ARM64 cross-compilation; native builds for everything else

## Implementation Plan

### Task Breakdown (High Level)

1. **CI Workflow** — Create `.github/workflows/ci.yml` with test, clippy, and fmt checks running on every PR/push to main. Include Rust caching.
2. **Release Versioning with release-please** — Configure release-please GitHub Action for automated version bumps, changelog generation, and tag creation from conventional commits.
3. **Cross-Platform Release Build Workflow** — Create `.github/workflows/release.yml` with build matrix for all 5 target platforms, producing both `cadre` and `cadre-mcp` binaries.
4. **Release Artifact Packaging** — Build a `scripts/package.sh` script that creates distributable archives (tar.gz/zip) with binaries, LICENSE, README, and SHA256 checksums. Integrate into release workflow.
5. **Makefile Enhancements** — Extend the existing Makefile with `ci`, `release-local`, and `package` targets for local developer workflows.
6. **Homebrew Tap** — Create `homebrew-cadre` tap repository with a formula for macOS distribution. Add a release workflow job to auto-update the formula on new releases.

### Dependencies Between Tasks
- CI Workflow (SMET-T-0139) is independent and can be done first
- Release-please (SMET-T-0140) is independent
- Release Build Workflow (SMET-T-0141) depends on release-please (SMET-T-0140) for tag triggers and packaging script (SMET-T-0142)
- Artifact Packaging (SMET-T-0142) is independent (script is standalone, consumed by others)
- Makefile Enhancements (SMET-T-0143) depends on packaging script (SMET-T-0142) for `make package` target
- Homebrew Tap (SMET-T-0144) depends on release workflow (SMET-T-0141) and packaging (SMET-T-0142)

### Recommended Execution Order
1. SMET-T-0139 (CI Workflow) — independent, immediate value
2. SMET-T-0140 (release-please) — independent, enables release flow
3. SMET-T-0142 (packaging script) — independent, needed by others
4. SMET-T-0143 (Makefile enhancements) — depends on packaging script
5. SMET-T-0141 (release build workflow) — depends on release-please + packaging script
6. SMET-T-0144 (Homebrew tap) — depends on release workflow being functional

### Total Estimated Effort
9-15 days across all 6 tasks

## Cadre ADR Alignment (SMET-A-0001)

**Audit date**: 2026-03-23 | **Recommendation**: Update scope (rename)

All 6 tasks completed. The rename (SMET-I-0074) will change:
- Binary names in CI/release workflows: `cadre` → `cadre`, `cadre-mcp` → `cadre-mcp`
- Artifact naming: `cadre-{version}-{target}.tar.gz` → `cadre-{version}-{target}.tar.gz`
- Homebrew tap: `homebrew-cadre` → `homebrew-cadre`
- Makefile targets reference new binary names

These are mechanical updates applied during the rename initiative.