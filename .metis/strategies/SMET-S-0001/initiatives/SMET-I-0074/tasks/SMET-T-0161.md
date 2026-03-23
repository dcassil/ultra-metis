---
id: rename-plugin-directory-and-update
level: task
title: "Rename plugin directory and update all plugin files"
short_code: "SMET-T-0161"
created_at: 2026-03-23T20:15:06.552348+00:00
updated_at: 2026-03-23T20:20:44.728835+00:00
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

# Rename plugin directory and update all plugin files

## Parent Initiative
[[SMET-I-0074]]

## Objective
Rename `plugins/ultra-metis/` directory to `plugins/cadre/`, rename all command files from `ultra-metis-*` to `cadre-*`, update all internal references in plugin.json, commands, hooks, skills, scripts, agents, and .mcp.json within the plugin. Update MCP tool prefixes from `mcp__ultra-metis__` to `mcp__cadre__` and slash commands from `/ultra-metis-*` to `/cadre-*`.

## Scope
- Plugin directory: plugins/ultra-metis/ → plugins/cadre/
- plugin.json: update name and all references
- Commands: ultra-metis-ralph.md → cadre-ralph.md, ultra-metis-decompose.md → cadre-decompose.md, ultra-metis-ralph-epic.md → cadre-ralph-epic.md
- Hooks: session-start-hook.sh, pre-compact-hook.sh (update internal references)
- Skills: all SKILL.md files referencing ultra-metis
- Scripts: setup-ultra-metis-ralph.sh → setup-cadre-ralph.sh, setup-ultra-metis-decompose.sh → setup-cadre-decompose.sh, check-dependencies.sh
- Agents: flight-levels.md
- Plugin .mcp.json

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] Plugin directory is plugins/cadre/
- [ ] All slash commands use /cadre-* naming
- [ ] All MCP tool references use mcp__cadre__ prefix
- [ ] All internal file references updated
- [ ] No "ultra-metis" references remain in plugin files

## Status Updates