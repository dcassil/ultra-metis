---
id: update-project-config-files-mcp
level: task
title: "Update project config files: .mcp.json, Makefile, CLAUDE.md, and default project directory"
short_code: "SMET-T-0162"
created_at: 2026-03-23T20:15:08.638677+00:00
updated_at: 2026-03-23T20:22:37.427795+00:00
parent: SMET-I-0074
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0074
---

# Update project config files: .mcp.json, Makefile, CLAUDE.md, and default project directory

## Parent Initiative
[[SMET-I-0074]]

## Objective
Update all project-level config files to reference the new Cadre naming. This includes the root .mcp.json (binary name and plugin path), Makefile (binary targets, install paths), CLAUDE.md (crate names, binary names, plugin references), and the default project directory constant in Rust init code (`.cadre/` instead of `.ultra-metis/`). Also update benchmarks scripts referencing ultra-metis.

## Scope
- Root .mcp.json: binary path and plugin directory references
- Makefile: all ultra-metis binary targets → cadre
- CLAUDE.md: crate names, build instructions, plugin references
- Rust init code: default project directory `.cadre/`
- benchmarks/run-ultra-metis-bench.sh → rename and update
- Any other top-level files referencing ultra-metis

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] .mcp.json references cadre-mcp binary and plugins/cadre
- [ ] Makefile builds cadre and cadre-mcp binaries
- [ ] CLAUDE.md accurately describes cadre-* crate layout
- [ ] Default project directory is .cadre/ for new projects
- [ ] Benchmark scripts use cadre naming

## Status Updates