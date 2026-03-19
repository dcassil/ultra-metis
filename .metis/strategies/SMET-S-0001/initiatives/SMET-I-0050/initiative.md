---
id: build-release-and-distribution
level: initiative
title: "Build, Release, and Distribution Strategy for Monorepo Artifacts"
short_code: "SMET-I-0050"
created_at: 2026-03-17T21:38:34.444414+00:00
updated_at: 2026-03-17T21:38:34.444414+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: build-release-and-distribution
---

# Build, Release, and Distribution Strategy for Monorepo Artifacts Initiative

## Context

Ultra-metis is now structured as a monorepo with multiple distinct components that serve very different deployment targets and user installation workflows:

| Component | Location | What it is | Where it goes |
|-----------|----------|------------|---------------|
| **ultra-metis-mcp** | `crates/ultra-metis-mcp/` | MCP server binary | Needs to be runnable by Claude Code via `.mcp.json` |
| **ultra-metis-cli** | `crates/ultra-metis-cli/` | CLI tool | Installed on user's PATH for terminal use |
| **ultra-metis-core** | `crates/ultra-metis-core/` | Shared Rust library | Compiled into binaries (not distributed standalone) |
| **ultra-metis-store** | `crates/ultra-metis-store/` | Persistence layer | Compiled into binaries (not distributed standalone) |
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
- npm package wrapper for `npx ultra-metis` style usage (future)
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

*To be filled during design phase after discovery questions are answered.*

## Alternatives Considered

*To be filled during design phase — expected alternatives include:*
- Cargo-only workflow (cargo install for everything)
- GitHub Releases with pre-built binaries
- Homebrew tap for macOS distribution
- Plugin as a standalone git repo vs subdirectory of monorepo
- Monorepo tooling: just vs make vs cargo-make vs nx

## Implementation Plan

*To be defined after design phase. Expected deliverables:*
1. Decision document on build/release tooling
2. Build scripts or Justfile for all components
3. Release workflow (manual or automated)
4. Installation documentation for each component
5. Plugin directory structure that Claude Code can consume