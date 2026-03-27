---
id: machine-runner-repo-discovery
level: task
title: "Machine Runner Repo Discovery"
short_code: "SMET-T-0200"
created_at: 2026-03-27T16:18:42.421306+00:00
updated_at: 2026-03-27T16:18:42.421306+00:00
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

# Machine Runner Repo Discovery

## Parent Initiative

[[SMET-I-0039]] — Machine Connectivity and Trust

## Objective

Implement the repo/working-directory discovery module in the Machine Runner. This module scans configured directories on the local machine to find git repositories and Cadre-managed projects, producing a list of repo name + path pairs that is sent to the Control Service during registration and heartbeat updates.

## Acceptance Criteria

- [ ] Repo discovery reads a list of scan directories from the Machine Runner config file (e.g., `repo_directories = ["~/projects", "~/work"]`)
- [ ] For each configured directory, scans one level deep for subdirectories that contain a `.git` folder (indicating a git repository)
- [ ] Each discovered repo produces a `RepoInfo` struct with: `repo_name` (directory name), `repo_path` (absolute path on disk)
- [ ] Optionally detects Cadre-managed repos by checking for a `.cadre/` or `.metis/` directory — adds a `cadre_managed: bool` field to `RepoInfo`
- [ ] Discovery is re-run on each heartbeat cycle so that newly cloned repos appear and deleted repos disappear automatically
- [ ] Handles missing or inaccessible directories gracefully — logs a warning and skips, does not crash the runner
- [ ] Handles symlinks: follows symlinked directories but does not recurse infinitely (track visited inodes or canonical paths)
- [ ] Discovery completes within a reasonable time (< 2 seconds for directories with up to 500 subdirectories)
- [ ] Unit tests covering: normal discovery, empty directories, missing directories, symlink handling, and Cadre-managed detection

## Implementation Notes

### Technical Approach
- Implement as a standalone module in `apps/machine-runner/src/discovery.rs` (or similar)
- Use `std::fs::read_dir` for directory scanning — no need for recursive walks since we only go one level deep
- For each entry in a scan directory, check if it is a directory and if it contains a `.git` subdirectory
- Resolve `~` in paths using the `dirs` crate or `std::env::var("HOME")`
- The `RepoInfo` struct should be serializable with serde for inclusion in registration and heartbeat API request bodies
- The discovery function signature: `fn discover_repos(scan_dirs: &[PathBuf]) -> Vec<RepoInfo>` — pure function, easy to test
- For the Cadre-managed check, look for `.cadre/` directory first, then `.metis/` as fallback

### Dependencies
- Machine Runner config file structure (established in SMET-T-0199) — this task reads the `repo_directories` config key
- No dependency on the Control Service — this is a purely local filesystem module

### Risk Considerations
- Large directories with many subdirectories could slow down discovery — the one-level-deep constraint mitigates this, but add a timeout/limit as a safety net
- Permissions errors on directories should be caught and logged, not propagated as panics
- On macOS, some directories (e.g., ~/Library) may trigger permission prompts — document which directories to configure and which to avoid

## Status Updates

*To be added during implementation*