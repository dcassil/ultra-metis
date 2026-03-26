---
id: cadre-store-module-decomposition
level: initiative
title: "cadre-store: Module Decomposition and File Organization"
short_code: "SMET-I-0086"
created_at: 2026-03-26T17:21:36.001173+00:00
updated_at: 2026-03-26T17:21:36.001173+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0001
initiative_id: cadre-store-module-decomposition
---

# cadre-store: Module Decomposition and File Organization Initiative

## Context

cadre-store is the file persistence layer for Cadre. It has only **4 source files** but one of them - `store.rs` - is **2,081 lines**, making it the second largest file in the entire workspace. The crate also has `code_index.rs` at 468 lines, `error.rs` at 109 lines, and `config.rs` at 104 lines.

### Current Structure

```
cadre-store/src/
├── lib.rs          (15 LOC - re-exports)
├── store.rs        (2,081 LOC - DocumentStore + AnyDocument enum with 15 variants)
├── code_index.rs   (468 LOC - CodeIndex, CodeIndexer, CodeSymbol, SymbolKind)
├── config.rs       (104 LOC - ProjectConfig)
└── error.rs        (109 LOC - StoreError)
```

### The store.rs Problem

`store.rs` contains:
1. **`AnyDocument` enum** with 15 variants (one per document type) - type erasure for heterogeneous storage
2. **Pattern-matching dispatch methods** on AnyDocument: `short_code()`, `title()`, `document_type()`, `phase()`, `parent_id()`, `created_at()`, `updated_at()`, `tags()`, etc. - each method has a 15-arm match
3. **`DocumentStore` struct** with all CRUD operations (create, read, list, search, edit, transition, archive, etc.)
4. **File I/O logic** - reading/writing markdown files with YAML frontmatter
5. **Path resolution** - converting short codes to file paths, handling directory structure
6. **Tests** for all of the above

Every time a new document type is added, ~15 match arms need updating across ~10 methods, all in the same file. This is the highest-maintenance file in the project.

### code_index.rs Analysis

At 468 lines, this file contains:
- `CodeSymbol` and `SymbolKind` types
- `CodeIndexer` with tree-sitter parsing logic
- `CodeIndex` storage type
- Tests

This is borderline but could benefit from splitting types from implementation.

## Goals & Non-Goals

**Goals:**
- Split `store.rs` (2,081 LOC) into focused modules targeting ~100 LOC each
- Organize the `AnyDocument` dispatch pattern to minimize per-variant boilerplate
- Split `code_index.rs` into types and implementation
- Each file should have one primary export
- Maintain the exact same public API from lib.rs

**Non-Goals:**
- Replacing AnyDocument with trait objects (different architectural decision)
- Adding new document type support
- Changing the file format (markdown + YAML frontmatter)
- Changing the public API

## Detailed Design

### Splitting store.rs into a Module Directory

Replace `store.rs` with a `store/` directory:

```
cadre-store/src/
├── lib.rs              (15 LOC - unchanged re-exports)
├── store/
│   ├── mod.rs          (~30 LOC - module declarations, re-export DocumentStore + AnyDocument)
│   ├── any_document.rs (~100 LOC - AnyDocument enum definition)
│   ├── dispatch.rs     (~200 LOC - impl AnyDocument dispatch methods, or use macro)
│   ├── document_store.rs (~80 LOC - DocumentStore struct + constructor)
│   ├── crud.rs         (~150 LOC - create, read, update operations)
│   ├── query.rs        (~100 LOC - list, search, filter operations)
│   ├── transition.rs   (~80 LOC - phase transition logic)
│   ├── archive.rs      (~60 LOC - archive operations)
│   ├── paths.rs        (~80 LOC - path resolution, short code to file path)
│   ├── frontmatter.rs  (~80 LOC - YAML frontmatter read/write helpers)
│   └── tests/
│       ├── mod.rs      (~10 LOC)
│       ├── crud_tests.rs    (~200 LOC)
│       ├── query_tests.rs   (~150 LOC)
│       └── transition_tests.rs (~100 LOC)
```

### Reducing AnyDocument Boilerplate with a Macro

The 15-arm match pattern repeated across ~10 methods is a maintenance burden. Introduce a dispatch macro:

```rust
// In dispatch.rs
macro_rules! dispatch_any_document {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            AnyDocument::Vision(doc) => doc.$method($($arg),*),
            AnyDocument::Initiative(doc) => doc.$method($($arg),*),
            AnyDocument::Task(doc) => doc.$method($($arg),*),
            // ... all variants
        }
    };
}

impl AnyDocument {
    pub fn short_code(&self) -> &str {
        dispatch_any_document!(self, short_code)
    }
    pub fn title(&self) -> &str {
        dispatch_any_document!(self, title)
    }
    // etc.
}
```

This keeps dispatch logic centralized and reduces per-method boilerplate from ~15 lines to ~1 line.

### Splitting code_index.rs

```
cadre-store/src/
├── code_index/
│   ├── mod.rs      (~15 LOC - re-exports)
│   ├── types.rs    (~60 LOC - CodeSymbol, SymbolKind)
│   ├── indexer.rs  (~120 LOC - CodeIndexer with tree-sitter)
│   ├── index.rs    (~80 LOC - CodeIndex storage)
│   └── tests.rs    (~200 LOC)
```

### Module Boundaries

- **`store/`**: Owns all file I/O and persistence. No domain logic (validation, transitions rules) - delegates to cadre-core.
- **`code_index/`**: Self-contained tree-sitter code indexing. No dependency on store operations.
- **`config.rs`**: Configuration loading. No dependency on store or code_index.
- **`error.rs`**: Error types. Leaf module with no internal dependencies.

## Alternatives Considered

1. **Trait objects instead of enum dispatch**: Would require `Box<dyn Document>` and dynamic dispatch. More flexible but loses exhaustive match checking and adds runtime cost. Not recommended for a type-safe Rust codebase.
2. **Leave as single file with regions/comments**: Doesn't solve the cognitive load or module boundary issues.
3. **Code generation for AnyDocument**: Over-engineered for 15 variants. The macro approach is simpler and idiomatic.

## Implementation Plan

1. **Create store/ module directory** (1 task): Move store.rs into store/mod.rs, split into sub-modules one at a time
2. **Implement dispatch macro** (1 task): Create the macro and refactor AnyDocument methods to use it
3. **Split code_index.rs** (1 task): Extract types, indexer, and tests into separate files
4. **Verification** (part of each task): `cargo test --workspace` after each step