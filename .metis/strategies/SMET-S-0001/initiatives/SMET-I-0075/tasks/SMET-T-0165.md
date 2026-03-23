---
id: vendor-superpowers-plugin-v5-0-5
level: task
title: "Vendor superpowers plugin v5.0.5 with fallback resolution in setup scripts"
short_code: "SMET-T-0165"
created_at: 2026-03-23T20:50:33.128896+00:00
updated_at: 2026-03-23T20:57:43.737963+00:00
parent: SMET-I-0075
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0075
---

# Vendor superpowers plugin v5.0.5 with fallback resolution in setup scripts

## Parent Initiative
[[SMET-I-0075]]

## Objective
Copy the full superpowers plugin (v5.0.5, 130 files, ~1MB) from the Claude plugin cache into `vendor/superpowers/` in this repo. Add a VERSION file tracking the pinned version. Update setup scripts (setup-cadre-ralph.sh, setup-cadre-decompose.sh) to try the installed plugin path first, then fall back to the vendored copy.

## Scope
- Copy `/Users/danielcassil/.claude/plugins/cache/claude-plugins-official/superpowers/5.0.5` to `vendor/superpowers/`
- Add `vendor/superpowers/VERSION` with content `5.0.5`
- Update `plugins/cadre/scripts/setup-cadre-ralph.sh` to resolve superpowers skills from installed path first, vendor path second
- Update `plugins/cadre/scripts/setup-cadre-decompose.sh` similarly
- Add `vendor/` to .gitignore? No — vendor is intentionally tracked so the plugin works without superpowers being installed

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] `vendor/superpowers/` contains full v5.0.5 plugin
- [ ] `vendor/superpowers/VERSION` contains `5.0.5`
- [ ] Setup scripts try installed superpowers first, fall back to vendor
- [ ] Plugin works even if superpowers is not installed as a Claude Code plugin

## Status Updates