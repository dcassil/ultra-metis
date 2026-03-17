---
name: Superpowers Integration Strategy
description: Decision to use superpowers plugin as execution engine for Super-Metis MVP, then selectively internalize later
type: project
---

Use superpowers plugin (approach 2 - integrate) as the execution discipline engine for building Super-Metis MVP. Every task/initiative uses superpowers' methodology: brainstorm → spec → plan → TDD → subagent execution → review → verification.

**Why:** Gets the total Super-Metis framework up and vetted fast. Dog-foods the Metis + Superpowers integration. Generates real usage data about what works and what doesn't.

**How to apply:**
- For every Super-Metis task, use superpowers skills (brainstorming, writing-plans, TDD, verification-before-completion)
- Metis manages work items and tracking (initiatives, tasks, Ralph loops)
- Superpowers provides execution discipline within each task
- After MVP is vetted, come back and do approach 1 (reimplement) or 3 (learn-from) for individual superpowers capabilities one at a time
- This means Super-Metis runner (SMET-I-0025) should eventually be able to invoke superpowers-style skills natively
