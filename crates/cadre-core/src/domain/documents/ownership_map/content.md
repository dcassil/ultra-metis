# {{ title }}

## Scope Definition

<!-- Precisely define what is owned. Use paths, package names, or component identifiers. -->
- **Type**: (repo / package / module / component / service / system area)
- **Identifier**: (exact name, path pattern, or glob)
- **Boundaries**: (where this scope ends and adjacent ownership begins)

## Owner

<!-- Identify the accountable owner. One owner per scope -- no shared ownership. -->
- **Name/Team**: (primary owner)
- **Contact**: (how to reach the owner for questions or incidents)
- **Backup**: (secondary contact when primary is unavailable)

## Responsibilities

<!-- What the owner is specifically accountable for within this scope. -->
- [ ] Code review and merge approval
- [ ] Incident response and on-call
- [ ] Documentation maintenance
- [ ] Dependency updates
- [ ] Architecture decision authority

## Delegation

<!-- Any responsibilities delegated to others. Each delegation should be explicit. -->
| Responsibility | Delegated To | Conditions | Expires |
|---------------|-------------|------------|---------|
| (what) | (whom) | (when delegation applies) | (date or "ongoing") |