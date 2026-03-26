# JavaScript CLI Tool

## Overview

Command-based Node.js CLI architecture. CLI argument parsing is strictly separated from core logic. Commands are thin wrappers that validate input and delegate to core modules. Suitable for developer tools, build scripts, and automation.

## Structure

Commands define the CLI interface (arguments, flags, help text) and delegate to core modules for actual logic. Core modules are framework-agnostic and testable without CLI infrastructure. Shared utilities handle I/O, formatting, and config.

## Dependency Rules

- Commands depend on core modules
- Core modules are self-contained and CLI-framework-agnostic
- Utils are shared helpers with no domain knowledge

## Anti-Patterns

- Business logic in command handlers
- Core modules depending on CLI framework
- Untestable I/O-heavy code in core