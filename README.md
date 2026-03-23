Cadre

Cadre is built on top of Metis. Metis by itself is already an incredibly powerful foundation for durable project memory, structured planning, and AI-assisted software work. This project is an attempt to extend that foundation into a more complete repo-native engineering operating system; some of these additions may prove valuable, some may need refinement, and part of the goal is to find out what actually works in practice.

What Cadre is trying to add

Cadre is aimed at software teams and AI agents working inside a single repo or monorepo, where planning, architecture, rules, quality, and execution all need to stay connected.

The main direction is:

keep Metis’s durable, repo-local model

preserve its strength as structured project memory

extend it with stronger engineering governance

make architecture, validation, and quality more explicit

improve traceability for both human and AI-driven work

persist only the knowledge that is actually worth keeping

Reference Architecture

The reference architecture makes architecture a first-class, durable control artifact for the repo. Instead of treating structure as implied or informal, the repo gets an explicit architecture definition that can guide planning, rules, and validation.

At a high level, it works like this:

during setup, the repo is matched to a catalog pattern or a local reference is derived

for brownfield repos, the system evaluates the current structure and quality before deciding what to capture or recommend

the selected or accepted architecture is persisted as the governing reference

that reference defines expected layers, boundaries, dependency direction, naming patterns, and testing shape

downstream rules and analysis expectations are seeded from that reference

when the actual repo diverges, the difference becomes visible and reviewable

So it gives you a stable architectural source of truth: more explicit than tribal knowledge, more durable than chat, and more actionable than a one-time design doc.

Expanded Planning Hierarchy

Cadre expands planning beyond Metis’s core hierarchy so product intent, larger capabilities, implementation slices, and execution work are more clearly separated.

At a high level, it works like this:

a ProductDoc defines the repo-level product intent and scope

Epics group major capabilities or larger engineering efforts

Stories break that work into meaningful delivery slices such as feature, bugfix, refactor, migration, or investigation

Tasks represent the execution-level work

each level links to the levels above and below it

planning stays durable and traceable from intent to implementation

So it gives you a richer planning model: clearer than a flatter work tree, more connected to product intent, and better suited for real engineering delivery.

Rules and Governance Layer

The rules and governance layer adds durable engineering controls around how work is performed. Instead of relying only on prompts or convention, important constraints become explicit repo artifacts.

At a high level, it works like this:

rules are stored as durable governed artifacts

rules can exist at multiple scopes such as platform, org, repo, package, component, or task

some rules are seeded from the selected reference architecture

protected rules cannot be casually changed during execution

rule changes go through a propose, review, approve or reject, and apply flow

approvals, constraints, and validation policies are persisted alongside the rules

So it gives you a governed execution layer: stronger than convention, more reviewable than prompt instructions, and better suited for trusted human and AI collaboration.

Quality Baselines and Records

The quality model adds durable historical quality tracking instead of treating validation as only a one-time pass or fail event.

At a high level, it works like this:

the system captures baseline outputs from deterministic tools such as lint, type-check, tests, coverage, dependency analysis, security, or complexity tools

those baselines are stored as durable artifacts

future runs are compared against prior baselines

regressions, improvements, breaches, and accepted overrides are recorded

architecture conformance and boundary integrity can be part of the quality picture

remediation work can be triggered when quality degrades or repeated issues appear

So it gives you a historical quality layer: more useful than isolated tool output, more durable than CI logs, and better for understanding whether the repo is actually improving or drifting.

Execution Records and Traceability

The execution record system adds a durable audit spine for meaningful work runs. Instead of only seeing that a task exists or that code changed, the system records the evidence around how the work happened.

At a high level, it works like this:

each meaningful run records the initiating artifact such as a Story or Task

the system records context sources, rules consulted, notes fetched, tools run, and validations performed

files touched, decisions made, escalations, overrides, and final disposition are captured

transition and decision records extend that history over time

artifacts are cross-linked so relationships stay queryable

the record persists after the chat or session ends

So it gives you an evidence-backed delivery history: more useful than scattered logs, more reviewable than chat transcripts, and better for trust, debugging, and handoff.

Notes System

The notes system is a lightweight layer of durable repo memory for small, reusable engineering insight that is worth keeping, but does not need to become a full formal document.

At a high level, it works like this:

notes are stored with scope such as repo, package, path, subsystem, or symbol

relevant notes are fetched when new work starts in that area

during or after the work, the agent scores whether each note was helpful, meh, or harmful

new notes can be proposed from confirmed findings

notes that are unused, stale, or frequently harmful become prune candidates

notes that conflict with architecture or current reality get flagged for human review

So it gives you a self-maintaining memory layer: lighter than formal docs, more durable than chat, and continuously shaped by actual use.

Gates and Autonomy Modes

The gates and autonomy model adds explicit control points around execution so AI work can be trusted at different levels of human involvement.

At a high level, it works like this:

workflows pass through major gates such as entry, context sufficiency, solution readiness, validation, escalation, and completion

each gate can require evidence before work proceeds

escalation triggers can fire on uncertainty, policy conflict, architecture mismatch, failing validation, or high-impact change

the system supports tight collaboration, mixed mode, and more autonomous execution

the selected mode changes what can proceed automatically and what requires approval

approvals and overrides are recorded as durable artifacts

So it gives you controlled autonomy: more flexible than all-or-nothing approval, safer than unconstrained automation, and easier to adapt to team trust levels.

Internal Reasoning vs Durable Persistence

Cadre tries to make a stronger distinction between what should stay internal to the model and what is worth promoting into durable repo memory.

At a high level, it works like this:

moment-to-moment reasoning stays internal by default

lightweight reusable findings can become notes

governing, cross-agent, risk-relevant, or audit-relevant information can become formal artifacts

promotion happens only when the information is confirmed or valuable enough to keep

temporary guesses, dead-end reasoning, and micro-decisions are not automatically persisted

the repo memory stays smaller, cleaner, and more intentional over time

So it gives you cleaner durable memory: more useful than dumping transcripts, more selective than saving everything, and better for long-term signal over noise.

Brownfield Architecture Evaluation

Cadre adds a more explicit brownfield model so existing repos can be brought under governance without pretending they started clean.

At a high level, it works like this:

during initialization, the existing repo structure is inspected

static analysis and structural signals are used to assess architectural quality and coherence

if the architecture is strong, the system matches it to a known pattern or derives a faithful local reference

if the architecture is weak, the system recommends a stronger target pattern and explains likely refactor impact

the user can accept the recommendation or explicitly keep the current architecture as the reference

no repo is left without a durable architecture reference

So it gives you a practical brownfield path: more honest than assuming every repo is clean, more structured than informal discovery, and better for bringing existing systems into a governed model.

Static-Tool-First Execution

Cadre aims to lean harder into deterministic tools wherever possible, using AI more for orchestration, interpretation, synthesis, and durable recording.

At a high level, it works like this:

when a tool can answer a question, the tool is preferred

when a tool can validate a condition, the tool result is preferred

AI selects which tools to use and how to interpret the outputs

deterministic outputs are stored or referenced in durable records when relevant

reasoning is still used for judgment and synthesis, not as a substitute for available evidence

plugin leverage is preferred over rebuilding commodity execution capability

So it gives you more grounded execution: less dependent on unsupported reasoning, more evidence-backed in practice, and better aligned with repeatable engineering work.

Monorepo Direction

Cadre is intended to grow toward better monorepo support than a single flat project model allows.

At a high level, it works like this:

the system can eventually detect when initialization is happening at a monorepo root

root-level context can be separated from package-level work

packages may carry different architecture profiles or rules

shared governance can still exist at the root where appropriate

cross-package work can be coordinated without collapsing everything into one planning space

package-local durable state can coexist with a lighter root layer

So it gives you a path toward real monorepo governance: more flexible than pretending every package is the same, more organized than one shared memory bucket, and better matched to full-stack repo reality.

Direction

Cadre is based on the belief that Metis already provides something genuinely strong: durable project memory, structured work management, and a foundation for AI-assisted engineering. The goal here is to build on that strength, add the layers that seem missing for governed software delivery, and test whether those additions actually make the system more useful in real repos.