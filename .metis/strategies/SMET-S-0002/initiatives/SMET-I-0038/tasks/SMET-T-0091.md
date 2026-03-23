---
id: create-monorepo-directory-scaffold
level: task
title: "Create monorepo directory scaffold with .gitkeep placeholders"
short_code: "SMET-T-0091"
created_at: 2026-03-17T21:08:08.542348+00:00
updated_at: 2026-03-17T21:11:38.747703+00:00
parent: SMET-I-0038
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0038
---

# Create monorepo directory scaffold with .gitkeep placeholders

## Parent Initiative

[[SMET-I-0038]]

## Objective

Create the full monorepo directory structure at the repo root, with all required subdirectories and `.gitkeep` files in empty placeholder directories. This is the first step before any files are moved.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Top-level dirs created: `apps/`, `crates/`, `packages/`, `infra/`, `docs/`, `scripts/`, `tests/`
- [ ] `apps/` subdirs: `control-web/`, `control-api/`, `machine-runner/`
- [ ] `crates/` subdirs: `cadre-agents/`, `cadre-events/`, `cadre-notes/`, `cadre-policy/` (placeholders for future crates)
- [ ] `packages/` subdirs: `shared-contracts/`, `ui/`, `config/`
- [ ] `infra/` subdirs: `docker/`, `k8s/`, `cloudflare/`, `tailscale/`
- [ ] `docs/` subdirs: `architecture/`, `product/`, `operations/`
- [ ] All empty placeholder dirs have `.gitkeep` files
- [ ] No existing files are moved or modified in this task

## Implementation Notes

### Technical Approach
Use `mkdir -p` to create all directories, then `touch .gitkeep` in each empty placeholder directory. The existing `cadre/` and `metis/` directories are untouched — those are handled in later tasks.

### Dependencies
None — this is the first task and has no dependencies.

## Status Updates

### 2026-03-17
Created all top-level monorepo directories and .gitkeep placeholders:
- `apps/`: control-web, control-api, machine-runner
- `crates/`: cadre-agents, cadre-events, cadre-notes, cadre-policy (placeholders)
- `packages/`: shared-contracts, ui, config
- `infra/`: docker, k8s, cloudflare, tailscale
- `docs/`: architecture, product, operations
- `scripts/`, `tests/`
All 19 .gitkeep files confirmed present. ✓ COMPLETE