---
id: create-cadre-setup-skill-with
level: task
title: "Create cadre-setup Skill with Guided Post-Init Flow"
short_code: "SMET-T-0194"
created_at: 2026-03-27T16:06:14.148308+00:00
updated_at: 2026-03-27T16:19:34.059295+00:00
parent: SMET-I-0094
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0094
---

# Create cadre-setup Skill with Guided Post-Init Flow

## Parent Initiative

[[SMET-I-0094]]

## Objective

Create a new `cadre-setup` skill in `plugins/cadre/skills/cadre-setup/` that orchestrates the guided post-initialization workflow. The skill chains MCP tools (analyze_project, evaluate_brownfield, query_architecture_catalog, create_document, index_code) into an interactive flow that guides the user through creating a ProductDoc and optionally a ReferenceArchitecture.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Skill directory created at `plugins/cadre/skills/cadre-setup/SKILL.md`
- [ ] Skill has proper YAML frontmatter with name, description, and trigger patterns
- [ ] Skill handles brownfield path: analyze_project → evaluate_brownfield → present findings → interactive ProductDoc creation → suggest ReferenceArchitecture → optional index_code + capture_quality_baseline
- [ ] Skill handles greenfield path: interactive Q&A → ProductDoc creation → architecture catalog browsing → ReferenceArchitecture
- [ ] Skill is human-in-the-loop: asks for confirmation at each major step
- [ ] Skill is registered and discoverable via `/cadre-setup`

## Implementation Notes

### Technical Approach

Create `plugins/cadre/skills/cadre-setup/SKILL.md` following the pattern of existing skills (look at `project-patterns/SKILL.md` and `help/SKILL.md` for the frontmatter format).

The skill content should instruct the agent to:

**Step 1: Assess the project**
- Call `analyze_project` (or read the init response if just initialized)
- Determine brownfield vs greenfield

**Step 2: Brownfield path**
- Call `evaluate_brownfield` with detected language and project type
- Call `query_architecture_catalog` to find matching patterns
- Present findings to user in a clear summary
- Ask: "Based on this analysis, let's create a ProductDoc. What is the product vision?"
- Use user's response + code analysis to create and populate a ProductDoc via `create_document`
- If catalog match found: "I found a matching architecture pattern [name]. Create a ReferenceArchitecture?" → create if confirmed
- Optionally: "Want me to index the codebase and capture a quality baseline?" → `index_code` + `capture_quality_baseline`

**Step 3: Greenfield path**
- Ask: "This is a new project. What problem are you solving?"
- Interactive Q&A to gather: product vision, target users, key features, technology choices
- Create and populate ProductDoc
- Browse architecture catalog based on stated technology → suggest ReferenceArchitecture

**Step 4: Wrap up**
- Summarize what was created (ProductDoc, ReferenceArchitecture, code index, baseline)
- Suggest next steps: "Create your first Epic to start planning work"

### Trigger Description
The skill description should trigger on: "initialize cadre", "set up cadre project", "cadre setup", "create product doc for this project", "analyze this project for cadre"

### Files to Create
- **Create**: `plugins/cadre/skills/cadre-setup/SKILL.md`

### Dependencies
- SMET-I-0093 must be complete (analyze_project tool exists) — it is

## Status Updates

*To be added during implementation*