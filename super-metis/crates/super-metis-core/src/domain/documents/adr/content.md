# {{ title }}

*This template includes sections for various types of architectural decisions. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

<!-- What is the issue motivating this decision? Describe the current situation, the forces at play, and why a decision is needed now.
Include: what's broken or missing, what constraints exist, and who is affected. -->

{What is the issue that motivates this decision? What is the current situation and its shortcomings?}

## Decision **[REQUIRED]**

<!-- State the decision clearly and specifically. This is WHAT was decided, not WHY.
Example: "We will use Tera as the template engine, embedded at compile time via `include_str!`."
One or two sentences — if longer, the decision may need to be broken into multiple ADRs. -->

{What is the change that we're proposing or have decided to make?}

## Rationale **[REQUIRED]**

<!-- WHY this option was chosen over alternatives. Link to the Alternatives Analysis below if applicable.
Include the key deciding factors: performance data, team expertise, license constraints, integration costs, etc. -->

{Why this option was chosen — the specific reasoning that made this the best available decision}

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

{Delete this section if there is only one viable option}

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| {Option 1 — chosen} | {Benefits} | {Drawbacks} | Low/Medium/High | {Estimate} |
| {Option 2} | {Benefits} | {Drawbacks} | Low/Medium/High | {Estimate} |
| {Option 3} | {Benefits} | {Drawbacks} | Low/Medium/High | {Estimate} |

## Consequences **[REQUIRED]**

<!-- What becomes easier or harder as a result of this decision? Be honest about trade-offs. -->

### Positive
- {Benefit 1 — what this decision enables or improves}
- {Benefit 2}

### Negative
- {Cost or drawback 1 — what becomes harder or more expensive}
- {Cost or drawback 2}

### Neutral
- {Side effect that is neither good nor bad but worth noting}

## Review Schedule **[CONDITIONAL: Temporary Decision]**

{Delete this section if the decision is intended to be permanent}

### Review Triggers
- {Condition that would cause us to revisit this decision — e.g., "If latency exceeds 500ms at scale"}
- {Another trigger — e.g., "If the chosen library is abandoned or relicensed"}

### Scheduled Review
- **Next Review Date**: {Date}
- **Review Criteria**: {What to evaluate at that time}
- **Sunset Date**: {When this decision expires if not renewed}

## Status

{Current status: Draft / Under Discussion / Decided / Superseded}
