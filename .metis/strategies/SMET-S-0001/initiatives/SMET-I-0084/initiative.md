---
id: workspace-linting-and-quality
level: initiative
title: "Workspace Linting and Quality Configuration"
short_code: "SMET-I-0084"
created_at: 2026-03-26T17:21:33.364334+00:00
updated_at: 2026-03-26T18:31:33.411482+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: workspace-linting-and-quality
---

# Workspace Linting and Quality Configuration Initiative

## Context

The Cadre workspace currently has **zero linting or formatting configuration** beyond Rust defaults. CI runs `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`, but with no custom rules. There is no `rustfmt.toml`, no `clippy.toml`, no `.shellcheckrc`, and no markdown linting. The 812 lines of bash in the plugin have no static analysis. This initiative establishes workspace-wide quality tooling as a foundation for the restructuring initiatives (SMET-I-0085 through SMET-I-0089).

### Current State

| Tool | Config File | Status |
|------|------------|--------|
| rustfmt | rustfmt.toml | **Missing** - uses all defaults |
| clippy | clippy.toml | **Missing** - only `-D warnings` in CI |
| shellcheck | .shellcheckrc | **Missing** - 8 bash scripts with no linting |
| markdownlint | .markdownlint.json | **Missing** - skills/commands are markdown |
| CI | .github/workflows/ci.yml | **Minimal** - test + clippy + fmt only |

### Codebase Scale
- **Rust**: 126 files, ~49K LOC across 4 crates
- **Bash**: 8 scripts, ~812 LOC in plugins/cadre/
- **Markdown**: 9 skill/command/agent files
- **JSON**: 3 config files (plugin.json, hooks.json, .mcp.json)

## Goals & Non-Goals

**Goals:**
- Add `rustfmt.toml` with workspace-wide formatting rules (max width, import grouping, etc.)
- Add `clippy.toml` with stricter-than-default lint rules targeting file size and complexity
- Add `.shellcheckrc` and integrate shellcheck into CI for all bash scripts
- Add clippy lints for module boundary enforcement (visibility rules, re-export hygiene)
- Enhance CI workflow to run shellcheck and enforce new lint rules
- Establish documented coding standards in CLAUDE.md or a CONTRIBUTING.md

**Non-Goals:**
- Refactoring existing code to comply (that's covered by SMET-I-0085 through SMET-I-0089)
- Adding markdownlint (low priority given markdown is mostly skills/docs, not code)
- Adding pre-commit hooks (keep it CI-enforced for now)

## Detailed Design

### 1. rustfmt.toml (Workspace Root)

Create `/Users/danielcassil/projects/ultra-metis/rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Module"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
```

Key rules:
- **max_width = 100**: Industry standard for Rust, prevents overly-wide lines
- **imports_granularity = "Module"**: Groups imports by module, reducing import noise
- **group_imports = "StdExternalCrate"**: Separates std, external, and crate imports

### 2. clippy.toml (Workspace Root)

Create `/Users/danielcassil/projects/ultra-metis/clippy.toml`:

```toml
# Enforce smaller functions - target ~100 LOC per file means functions should be focused
too-many-lines-threshold = 80
too-many-arguments-threshold = 7
type-complexity-threshold = 250
cognitive-complexity-threshold = 25

# Module-level complexity limits
single-char-binding-names-threshold = 4
```

Additionally, add workspace-level clippy configuration via `Cargo.toml` or `lib.rs` attributes:

```rust
// In each crate's lib.rs or main.rs:
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]  // Common in domain types
#![warn(clippy::too_many_lines)]
#![warn(clippy::cognitive_complexity)]
```

Key clippy lints to enable:
- **`clippy::too_many_lines`**: Flags functions exceeding threshold (default 100, set to 80)
- **`clippy::cognitive_complexity`**: Flags overly complex functions
- **`clippy::large_enum_variant`**: Important for AnyDocument enum in cadre-store
- **`clippy::module_inception`**: Warns when mod.rs re-exports a type with the same name as the module
- **`clippy::wildcard_imports`**: Prevents `use foo::*` that breaks module boundaries
- **`clippy::pub_use`** (nursery): Audit re-exports in lib.rs

### 3. .shellcheckrc (Workspace Root)

Create `/Users/danielcassil/projects/ultra-metis/.shellcheckrc`:

```
# Default shell for all scripts
shell=bash

# Enable all checks by default, selectively disable
# SC2086: Double quote to prevent globbing - allow in controlled cases
# SC1091: Can't follow non-constant source - common in plugin scripts
disable=SC1091
```

### 4. CI Enhancements (.github/workflows/ci.yml)

Add to existing CI workflow:

```yaml
shellcheck:
  name: Shell Script Lint
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run shellcheck
      uses: ludeeus/action-shellcheck@2.0.0
      with:
        scandir: './plugins'
        severity: warning

clippy-strict:
  name: Strict Clippy (Pedantic)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - uses: Swatinem/rust-cache@v2
    - name: Run strict clippy
      run: cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic
```

### 5. Makefile Enhancements

Add targets:

```makefile
lint-shell:
	shellcheck plugins/cadre/**/*.sh plugins/cadre/hooks/*.sh

lint-strict:
	cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic

ci: test lint lint-shell fmt-check
```

### 6. Module Boundary Rules

Establish and document these rules (enforceable via clippy lints and code review):

- **No `pub use *` (wildcard re-exports)**: Use explicit re-exports in lib.rs
- **One primary type per file**: Each .rs file should export one primary struct/enum/trait
- **Tests in separate files**: `#[cfg(test)] mod tests` should be in a sibling `tests.rs` file when the parent exceeds 100 LOC
- **Private by default**: Only `pub` what's needed by other crates; use `pub(crate)` for internal sharing
- **Crate boundaries**: cadre-core exposes domain types; cadre-store handles persistence; cadre-cli/cadre-mcp are consumers only

## Alternatives Considered

1. **Pre-commit hooks instead of CI**: Rejected. CI enforcement is more reliable and doesn't slow local development. Pre-commit can be added later as opt-in.
2. **cargo-deny for dependency auditing**: Worth adding but out of scope - this initiative focuses on code quality, not supply chain.
3. **Nightly-only rustfmt features**: Some rustfmt options (like `imports_granularity`) require nightly. If stable doesn't support a feature, skip it rather than requiring nightly.

## Implementation Plan

1. Create rustfmt.toml and verify `cargo fmt --all -- --check` still passes
2. Create clippy.toml with thresholds; run clippy to baseline current violations (don't fix yet)
3. Add `#![warn(...)]` attributes to each crate, initially allowing known violations with `#![allow(...)]` per-crate
4. Create .shellcheckrc and run shellcheck on all bash scripts; fix any issues
5. Update CI workflow with shellcheck job
6. Update Makefile with new lint targets
7. Document coding standards in project documentation