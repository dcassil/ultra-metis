# Python CLI Tool (Click/Typer)

## Overview

Command-based CLI architecture using Click or Typer for argument parsing. Commands are thin wrappers that validate input and delegate to core library modules. Core logic is testable without CLI infrastructure. Suitable for developer tools, data pipelines, automation scripts, and system utilities.

## Structure

CLI entry point defines command groups and subcommands using decorators. Each command validates parsed arguments and delegates to core modules. Core modules contain pure business logic with no CLI framework dependencies. Utils provide shared helpers for I/O, formatting, and configuration.

## Dependency Rules

- CLI commands depend on core modules only
- Core modules have no CLI framework imports (no click/typer)
- Core modules use dependency injection for I/O (file handles, streams)
- Utils are shared helpers with no domain knowledge

## Anti-Patterns

- Business logic in click/typer command functions
- Core modules importing click or typer
- Hardcoded file paths or stdin/stdout in core logic
- Missing --help text on commands and options
- Monolithic single-file CLI with all logic inline

## Quality Expectations

- Ruff or flake8 clean with no warnings
- Type hints on all public functions
- Core library has unit tests independent of CLI
- CLI integration tests using CliRunner or subprocess