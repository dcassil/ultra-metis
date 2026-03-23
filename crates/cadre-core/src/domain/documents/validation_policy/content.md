# {{ title }}

## Policy Scope

<!-- Define precisely which artifacts and transitions this policy governs. -->
- **Document Types**: (e.g., "story:feature", "task:*", "epic:*")
- **Transitions**: (e.g., "active -> completed", "ready -> active")
- **Applies When**: (conditions under which this policy activates)

## Required Validations

<!-- Each validation MUST pass before the transition is allowed. Be specific about what each checks. -->
| Validation | What It Checks | Tool/Method | Failure Behavior |
|------------|---------------|-------------|-----------------|
| (name) | (description) | (tool or manual) | blocking / advisory |

## Optional Validations

<!-- Recommended but non-blocking checks. Failures produce warnings, not blocks. -->
| Validation | What It Checks | Why Recommended |
|------------|---------------|-----------------|
| (name) | (description) | (value it provides) |

## Exceptions

<!-- How to request a waiver. Reference the GateOverride workflow. -->
- **Process**: (how to request an exception -- typically via GateOverride document)
- **Authority**: (who can grant exceptions)
- **Known Permanent Exceptions**: (list any, with justification)