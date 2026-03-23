---
id: rename-cadre-ralph-epic-to-cadre
level: task
title: "Rename cadre-ralph-epic to cadre-ralph-initiative and update command/script references"
short_code: "SMET-T-0167"
created_at: 2026-03-23T21:11:13.230892+00:00
updated_at: 2026-03-23T21:13:51.431645+00:00
parent: SMET-I-0076
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0076
---

# Rename cadre-ralph-epic to cadre-ralph-initiative and update command/script references

## Parent Initiative
[[SMET-I-0076]]

## Objective
Rename the `/cadre-ralph-epic` command to `/cadre-ralph-initiative` to match the Cadre (Metis) terminology (initiatives, not epics). Rename the command file and update all internal references. Also update the short code validation to accept initiative codes (PREFIX-I-NNNN) instead of epic codes (PREFIX-E-NNNN).

## Scope
- `git mv` commands/cadre-ralph-epic.md → commands/cadre-ralph-initiative.md
- Update command content: "epic" → "initiative", "E-NNNN" → "I-NNNN"
- Update any references in other commands, skills, or CLAUDE.md
- Update the setup script that cadre-ralph-epic called (if separate from cadre-ralph)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] `/cadre-ralph-initiative` command exists and is discoverable
- [ ] Old `/cadre-ralph-epic` command removed
- [ ] Command accepts initiative short codes (PREFIX-I-NNNN)
- [ ] All internal references updated

## Status Updates