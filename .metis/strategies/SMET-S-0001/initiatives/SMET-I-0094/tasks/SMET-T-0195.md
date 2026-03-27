---
id: enhance-sessionstart-hook-to
level: task
title: "Enhance SessionStart Hook to Detect Incomplete Initialization"
short_code: "SMET-T-0195"
created_at: 2026-03-27T16:06:15.484567+00:00
updated_at: 2026-03-27T16:19:34.450034+00:00
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

# Enhance SessionStart Hook to Detect Incomplete Initialization

## Parent Initiative

[[SMET-I-0094]]

## Objective

Modify `plugins/cadre/hooks/session-start-hook.sh` to detect the "initialized but no ProductDoc" state and prompt the user to run `/cadre-setup`. This ensures users who initialized a project but didn't complete the guided setup are reminded on their next session.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] SessionStart hook detects `.cadre/` exists but no ProductDoc documents
- [ ] Hook outputs a clear "Setup Incomplete" message with `/cadre-setup` suggestion
- [ ] Message does not appear when a ProductDoc already exists
- [ ] Message does not appear when `.cadre/` doesn't exist
- [ ] Existing hook behavior is preserved (active work items, tool reference, etc.)

## Implementation Notes

### Technical Approach

Add a check after the existing `.cadre/` detection in `session-start-hook.sh`:

```bash
# After existing cadre status check...
# Check for incomplete setup (initialized but no ProductDoc)
PRODUCT_DOC_COUNT=$(cadre list --type product_doc 2>/dev/null | grep -c "product_doc" || true)
if [ "$PRODUCT_DOC_COUNT" -eq 0 ]; then
  echo ""
  echo "## Setup Incomplete"
  echo "This project has been initialized but has no ProductDoc."
  echo "Run \`/cadre-setup\` to complete the guided project setup."
  echo ""
fi
```

The check uses `cadre list --type product_doc` to see if any ProductDoc documents exist. If zero, show the prompt. The `|| true` ensures the script doesn't fail if the command errors.

### Files to Modify
- **Modify**: `plugins/cadre/hooks/session-start-hook.sh`

### Dependencies
- T-0194 (cadre-setup skill) should be done so `/cadre-setup` actually works when suggested

## Status Updates

*To be added during implementation*