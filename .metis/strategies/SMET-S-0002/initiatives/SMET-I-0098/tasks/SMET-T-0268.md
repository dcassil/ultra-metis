---
id: local-security-enforcement-and
level: task
title: "Local Security Enforcement and Session Approval Dialog"
short_code: "SMET-T-0268"
created_at: 2026-03-28T16:52:40.278229+00:00
updated_at: 2026-03-28T16:52:40.278229+00:00
parent: SMET-I-0098
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: SMET-S-0002
initiative_id: SMET-I-0098
---

# Local Security Enforcement and Session Approval Dialog

## Parent Initiative

[[SMET-I-0098]] — Installable Machine Runner with System Tray UI

## Objective

Implement local security enforcement from the desktop settings and the native session approval dialog. When `local_approval_required` is enabled, the runner must pop up a native OS dialog before starting any session.

## Acceptance Criteria

- [ ] Runner checks local settings before accepting any session command: validates autonomy level against `allowed_autonomy_levels`, checks repo against `allowed_repos`/`blocked_repos`, checks action categories against local overrides
- [ ] If `local_approval_required` is true: when a `start_session` command arrives, runner pauses execution and emits a Tauri event requesting local approval
- [ ] Tauri app shows a native notification + dialog: "Remote session requested: '{title}' in {repo}. Autonomy: {level}. Allow / Deny"
- [ ] User clicks Allow → runner proceeds with session start. Deny → runner rejects, reports `failed` to control service with `{"reason": "local_user_denied"}`
- [ ] If `session_timeout_minutes` > 0: runner spawns a watchdog timer that kills sessions exceeding the timeout
- [ ] If `block_autonomous_mode` is true: runner rejects `autonomous` sessions regardless of server policy
- [ ] If `restrict_to_repos` is true: runner only accepts sessions for repos in `allowed_repos` list
- [ ] All local enforcement logs violations to the runner's local log
- [ ] Unit tests for each enforcement check

## Implementation Notes

### Technical Approach
- Local enforcement runs in the runner core before the supervisor starts the process — it's a pre-check layer
- Session approval dialog uses Tauri's dialog API (`tauri::api::dialog`) for native OS dialogs, or a custom window for richer UI
- Timeout watchdog: `tokio::time::sleep(duration)` task that calls `supervisor.force_stop_session()` when it fires
- Local enforcement is additive to server-side policy (SMET-I-0044) — both must pass

### Dependencies
- SMET-T-0264 (Runner Library API), SMET-T-0266 (Settings — security settings)

## Status Updates

*To be added during implementation*