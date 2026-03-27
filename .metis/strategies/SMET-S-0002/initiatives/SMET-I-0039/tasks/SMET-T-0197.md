---
id: machine-data-model-and-database
level: task
title: "Machine Data Model and Database Schema"
short_code: "SMET-T-0197"
created_at: 2026-03-27T16:18:36.490306+00:00
updated_at: 2026-03-27T16:18:36.490306+00:00
parent: SMET-I-0039
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0039
---

# Machine Data Model and Database Schema

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Define and implement the core data model and database schema for machines, repos, and the multi-tenancy scaffolding tables (orgs, teams, users, roles). This is the foundational data layer that all other tasks in this initiative depend on.

## Acceptance Criteria

- [ ] `machines` table created with columns: `id` (UUID PK), `name` (text, not null), `platform` (text), `status` (enum: pending/trusted/revoked), `trust_tier` (enum: trusted/restricted), `last_heartbeat` (timestamp nullable), `user_id` (FK), `team_id` (FK), `org_id` (FK), `metadata` (JSONB), `created_at`, `updated_at`
- [ ] `machine_repos` table created with columns: `id` (UUID PK), `machine_id` (FK to machines), `repo_name` (text), `repo_path` (text), `created_at`, `updated_at`
- [ ] Multi-tenancy seed tables created: `orgs` (id, name), `teams` (id, org_id, name), `users` (id, team_id, org_id, name, email), `roles` (id, name, permissions_json), `user_roles` (user_id, role_id)
- [ ] Database migration seeds a default org, team, user, and "default" role with empty permissions JSON
- [ ] Rust domain types defined for Machine, MachineRepo, MachineStatus (enum), TrustTier (enum) with serde serialization
- [ ] Machine status computation helper: given `last_heartbeat` timestamp, returns online (< 30s ago), stale (30s-5m ago), or offline (> 5m ago)
- [ ] Unit tests for domain types, status computation, and serialization round-trips

## Implementation Notes

### Technical Approach
- Create a SQL migration file (using the project's migration tooling) that defines all tables with proper foreign key constraints and indexes
- Define Rust structs in the control-api data layer: `Machine`, `MachineRepo`, `MachineStatus`, `TrustTier`
- `MachineStatus` is a computed field, not stored — derived from `last_heartbeat` relative to current time
- Trust tier is stored on the machine record and defaults to `trusted` when a machine is approved
- Seed migration inserts: `org(id=1, name="default")`, `team(id=1, org_id=1, name="default")`, `user(id=1, team_id=1, org_id=1, name="default", email="admin@localhost")`, `role(id=1, name="default", permissions_json="{}")`, `user_role(user_id=1, role_id=1)`
- Index on `machines(user_id)` for scoped queries, index on `machine_repos(machine_id)` for joins

### Dependencies
- SMET-I-0095 (Monorepo Restructure) must be complete — `apps/control-api/` directory must exist
- Database infrastructure and migration tooling must be set up

### Risk Considerations
- Schema changes are hard to undo once other tasks build on them — review carefully before merging
- The multi-tenancy columns (user_id, team_id, org_id) are all non-null with defaults to support MVP single-user mode while being ready for real multi-tenancy later

## Status Updates

*To be added during implementation*