# Rust CLI Tool (Clap)

## Overview

Command-based CLI architecture using Clap for argument parsing. Commands are thin wrappers that validate input and delegate to library logic in a separate crate or module. Core logic is testable without CLI infrastructure. Suitable for developer tools, system utilities, and automation.

## Structure

The binary crate handles argument parsing with Clap derive macros and delegates to a library crate (or lib.rs) for all business logic. Commands map to subcommand enums. Each command module validates parsed arguments and calls into core. Core modules are pure functions and structs with no CLI framework dependencies.

## Dependency Rules

- Binary crate depends on library crate
- Command handlers depend on core modules only
- Core modules are CLI-framework-agnostic
- Core modules use trait objects for I/O (readers, writers, filesystem)
- Error types use thiserror for library, anyhow for binary

## Anti-Patterns

- Business logic in main.rs or command match arms
- Core modules importing clap types
- Untestable I/O (hardcoded stdin/stdout/filesystem paths in core)
- Using anyhow in library code instead of typed errors
- Monolithic main.rs with all logic inline

## Quality Expectations

- Clippy clean with no warnings
- All public API documented with rustdoc
- Core library has unit tests independent of CLI
- Integration tests exercise full CLI via assert_cmd or similar