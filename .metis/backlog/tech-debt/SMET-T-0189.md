---
id: audit-and-replace-remaining-metis
level: task
title: "Audit and Replace Remaining Metis References with Cadre Equivalents"
short_code: "SMET-T-0189"
created_at: 2026-03-27T15:48:26.003940+00:00
updated_at: 2026-03-27T15:48:26.003940+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Audit and Replace Remaining Metis References with Cadre Equivalents

## Objective

Audit the entire codebase for stale references to the old "metis" naming that should now use "cadre" equivalents. This includes Rust source code, plugin configs, MCP tool prefixes, string literals, comments, and documentation. Replace all incorrect references with the correct cadre versions.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: After the metis→cadre rename (SMET-I-0074), some references to the old naming may remain — particularly in string literals, MCP tool name prefixes (`mcp__cadre__` vs `mcp__cadre__`), plugin registration, and comments.
- **Benefits of Fixing**: Consistent naming throughout, avoids confusion when onboarding or reading code, prevents bugs from mismatched tool name prefixes.
- **Risk Assessment**: Low risk if left unaddressed short-term, but stale names compound confusion over time.

## Acceptance Criteria

- [ ] `rg -i metis` across `crates/` returns zero hits outside of `.metis/` directory references and the Metis plugin integration (which is the upstream plugin, not our code)
- [ ] All MCP tool name prefixes in Cadre source code use `mcp__cadre__` not `mcp__cadre__`
- [ ] All user-facing strings (error messages, help text, CLI output) reference "cadre" not "metis"
- [ ] Plugin config files (`.mcp.json`, `plugin.json`) use correct cadre binary names and prefixes
- [ ] Comments and doc strings updated where they reference the old name
- [ ] `cargo test --workspace` passes after all replacements
- [ ] `make build && make install` succeeds

## Implementation Notes

### Technical Approach

Search patterns to audit:
- `rg -i "metis" crates/` — Rust source code
- `rg -i "mcp__metis" .` — MCP tool prefix references
- `rg -i "metis_plugin" .` — Plugin registration references
- `rg "metis" plugins/cadre/` — Plugin shell scripts and configs
- Check `.mcp.json`, `Makefile`, `CLAUDE.md` for stale references

Exclude from changes:
- `.metis/` directory (this is the Metis data store, not our code — its naming is correct)
- References to the upstream Metis plugin itself (e.g., `plugin:cadre:metis` MCP server name)
- Benchmark code that intentionally compares Metis vs Cadre (e.g., comparative execution modes, scoring reports)
- Original Metis source kept as reference material
- Git history / commit messages

### Dependencies
- None

## Status Updates

*To be added during implementation*