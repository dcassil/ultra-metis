---
id: cadre-cli-module-decomposition
level: initiative
title: "cadre-cli: Module Decomposition from Monolithic main.rs"
short_code: "SMET-I-0087"
created_at: 2026-03-26T17:21:36.933622+00:00
updated_at: 2026-03-26T17:21:36.933622+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: cadre-cli-module-decomposition
---

# cadre-cli: Module Decomposition from Monolithic main.rs Initiative

## Context

cadre-cli is the command-line interface for Cadre. It is a **single `main.rs` file at 2,019 lines** - the third largest file in the workspace. This monolithic binary contains all CLI argument parsing (via clap derive macros), all command handler implementations, output formatting, and error handling in one file.

### Current Structure

```
cadre-cli/src/
└── main.rs    (2,019 LOC - EVERYTHING)
```

### What's Inside main.rs

1. **Clap CLI definition** (~200 LOC): `#[derive(Parser)]` struct with `#[command(subcommand)]` enum listing ~20 subcommands
2. **Subcommand enums** (~300 LOC): Each subcommand's argument struct with clap attributes
3. **Main dispatch function** (~50 LOC): Match on subcommand, call handler
4. **Command handlers** (~1,200 LOC): 20 handler functions, each 30-100 lines
5. **Output formatting** (~200 LOC): Table formatting, color output helpers
6. **Error handling** (~70 LOC): Result conversion and user-facing error display

### Commands Currently Implemented

| Command | Approximate Handler LOC |
|---------|------------------------|
| init | ~80 |
| list | ~60 |
| read | ~40 |
| create | ~80 |
| edit | ~50 |
| transition-phase | ~40 |
| search-docs | ~50 |
| compare-policies | ~60 |
| list-policies | ~40 |
| create-policy | ~50 |
| list-rules | ~40 |
| trace-ancestry | ~50 |
| apply-coverage-policy | ~60 |
| apply-eslint-policy | ~60 |
| query-catalog | ~80 |
| evaluate-brownfield | ~80 |
| capture-quality-baseline | ~60 |
| capture-baseline | ~50 |
| compare-baselines | ~60 |
| index-code | ~40 |

## Goals & Non-Goals

**Goals:**
- Split monolithic `main.rs` (2,019 LOC) into focused command modules targeting ~100 LOC each
- Group commands by domain (document management, quality, catalog, etc.)
- Each command handler in its own file with one primary export
- Shared CLI utilities (output formatting, error display) in a dedicated module
- Keep clap argument structs near their handlers for co-location

**Non-Goals:**
- Adding new CLI commands
- Changing command behavior or output format
- Adding a library interface (cadre-cli stays a binary-only crate)

## Detailed Design

### Target Module Structure

```
cadre-cli/src/
├── main.rs             (~30 LOC - entry point, calls cli::run())
├── cli.rs              (~60 LOC - top-level Clap Parser, subcommand enum, dispatch)
├── output.rs           (~80 LOC - table formatting, color helpers, shared display)
├── error.rs            (~50 LOC - CLI-specific error display)
├── commands/
│   ├── mod.rs          (~20 LOC - re-exports all command modules)
│   ├── init.rs         (~80 LOC - init command)
│   ├── documents.rs    (~100 LOC - list, read, create, edit, search commands)
│   ├── transitions.rs  (~60 LOC - transition-phase command)
│   ├── quality.rs      (~100 LOC - capture-baseline, compare-baselines, apply-*-policy)
│   ├── rules.rs        (~60 LOC - list-rules, create-policy, compare-policies, list-policies)
│   ├── catalog.rs      (~80 LOC - query-catalog, evaluate-brownfield)
│   ├── tracing.rs      (~50 LOC - trace-ancestry)
│   └── code.rs         (~40 LOC - index-code)
```

### Splitting Strategy

**Phase 1: Extract shared utilities**
- Move output formatting functions to `output.rs`
- Move error handling to `error.rs`

**Phase 2: Extract the CLI definition**
- Move the `#[derive(Parser)]` struct and subcommand enum to `cli.rs`
- Keep `main.rs` as just `fn main() { cli::run() }`

**Phase 3: Extract command handlers by domain**
- Group related commands into domain modules under `commands/`
- Each module gets the clap subcommand structs for its commands plus the handler functions
- The `cli.rs` dispatch function calls into command modules

### Clap Organization Pattern

Use nested subcommand enums to group related commands:

```rust
// cli.rs
#[derive(Parser)]
#[command(name = "cadre")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(commands::init::InitArgs),
    List(commands::documents::ListArgs),
    Read(commands::documents::ReadArgs),
    Create(commands::documents::CreateArgs),
    // ...
}
```

Each command module exports its args struct and handler:

```rust
// commands/init.rs
#[derive(clap::Args)]
pub struct InitArgs { /* ... */ }

pub fn run(args: InitArgs, store: &DocumentStore) -> Result<()> { /* ... */ }
```

### Module Boundaries

- **`commands/`**: Each module handles one domain. Depends on cadre-core types and cadre-store DocumentStore.
- **`output.rs`**: Pure formatting functions. No dependencies on commands.
- **`error.rs`**: Maps anyhow/store errors to user-friendly output. No dependencies on commands.
- **`cli.rs`**: Owns argument parsing and dispatch. Depends on all command modules.
- **`main.rs`**: Entry point only. Calls `cli::run()`.

## Alternatives Considered

1. **One file per command (20+ files)**: Too granular for commands under 50 LOC. Domain grouping keeps related commands together while staying under ~100 LOC per file.
2. **Keep as single file with `mod` blocks**: Doesn't improve navigation or module boundaries.
3. **Convert to library + thin binary**: Over-engineering for a CLI that's only consumed as a binary.

## Implementation Plan

1. **Extract shared utilities** (1 task): Move output formatting and error handling to separate files
2. **Extract CLI definition** (1 task): Move Parser struct and subcommand enum to cli.rs, thin out main.rs
3. **Extract command handlers** (1 task): Move each command group to commands/ directory
4. **Verification** (part of each task): `cargo build -p cadre-cli && cargo test -p cadre-cli` after each step