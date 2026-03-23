---
id: claude-code-plugin-installation
level: initiative
title: "Claude Code Plugin Installation and Local Development Setup"
short_code: "SMET-I-0058"
created_at: 2026-03-18T17:08:34.782930+00:00
updated_at: 2026-03-18T17:39:05.234909+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0001
initiative_id: claude-code-plugin-installation
---

# Claude Code Plugin Installation and Local Development Setup Initiative

## Context

Cadre has a working MCP server (`cadre-mcp`) and CLI (`cadre`), but neither can be easily installed as a Claude Code plugin today. The current `plugin.json` uses `cargo run` to launch the MCP server, which:

- **Requires Rust toolchain** on the user's machine
- **Slow startup** — cargo checks for recompilation on every launch
- **Only works from repo directory** — paths are relative to the workspace

This initiative handles the immediate work to make cadre installable via `claude plugin add` for local development and dogfooding. Broader cross-platform distribution, CI/CD, and marketplace publishing are tracked in [[SMET-I-0050]].

## Current State

**plugin.json** (current):
```json
{
  "mcpServers": {
    "cadre": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "${pluginDir}/Cargo.toml", "-p", "cadre-mcp", "--release", "--"]
    }
  }
}
```

**Binary exists**: `target/release/cadre-mcp` (8.1 MB, macOS arm64) — already built, just not referenced.

## Goals & Non-Goals

**Goals:**
- Update `plugin.json` to reference the pre-built binary instead of `cargo run`
- Make the plugin installable via `claude plugin add /path/to/cadre`
- Provide a build script or Makefile target to rebuild the binary
- Ensure the MCP server starts fast and works reliably from the plugin directory
- Document the local install process

**Non-Goals:**
- Cross-platform binary distribution (SMET-I-0050)
- CI/CD pipeline (SMET-I-0050)
- Homebrew, npm, or marketplace publishing (SMET-I-0050)
- Install script for end users (SMET-I-0050)

## Detailed Design

### 1. Build release binary
Run `cargo build --release -p cadre-mcp` to produce `target/release/cadre-mcp`.

### 2. Update plugin.json
Change MCP server command from `cargo run` to the built binary. Use `${pluginDir}` to keep it portable within the plugin directory:

```json
{
  "mcpServers": {
    "cadre": {
      "command": "${pluginDir}/target/release/cadre-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

### 3. Add build convenience
Add a `Makefile` or `justfile` target so rebuilding after code changes is simple:
```
make build   # cargo build --release
make install # claude plugin add .
```

### 4. Verify installation
Test the full flow: `claude plugin add .` → launch Claude Code → verify MCP tools are available and responding.

## Alternatives Considered

1. **Keep `cargo run` approach** — rejected: too slow, requires Rust toolchain
2. **Symlink binary to PATH** — unnecessary complexity; `${pluginDir}` relative path works
3. **Separate plugin repo** — premature; monorepo plugin directory is fine for now

## Implementation Plan

1. Build release binary and verify it runs standalone
2. Update plugin.json to reference binary
3. Add build/install convenience targets
4. Test `claude plugin add` end-to-end
5. Document in README or CLAUDE.md