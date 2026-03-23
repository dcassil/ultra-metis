# {{ title }}

## Failed Gates

<!-- Detail every gate that was bypassed. Be precise about the gap between expected and actual. -->
| Gate | Metric | Threshold | Actual Value | Severity | Gap |
|------|--------|-----------|-------------|----------|-----|
| (gate name) | (metric) | (required value) | (actual value) | (blocking/advisory) | (how far off) |

## Override Justification

<!-- This is the core of the override record. Must be compelling enough to justify bypassing quality controls. -->
- **Business Context**: (why the transition cannot wait for gate compliance)
- **Urgency**: (what happens if we do NOT override)
- **Compensating Controls**: (what alternative safeguards are in place)
- **Remediation Plan**: (when and how the failed gates will be addressed -- reference a RemediationRecord if created)

## Approval Chain

<!-- Full audit trail of who approved this override and when. -->
| Approver | Role | Decision | Timestamp |
|----------|------|----------|-----------|
| (name) | (role) | approved/rejected | (when) |

- **Override Type**: (emergency / planned / exception)
- **Post-Hoc Review Required**: (yes/no -- if emergency override, when will review happen)