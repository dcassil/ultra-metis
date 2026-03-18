---
description: "Cancel active Ultra-Metis Ralph loop"
---

# Cancel Ultra-Metis Ralph

Cancel the active ralph loop by invoking the ralph-loop plugin's cancel command.

Use the Skill tool to invoke: `ralph-loop:cancel-ralph`

After cancellation, note that:
- The ralph-loop state file has been removed
- Any active Ultra-Metis document remains in its current phase (not reverted)
- You can resume work on the document manually or start a new loop
