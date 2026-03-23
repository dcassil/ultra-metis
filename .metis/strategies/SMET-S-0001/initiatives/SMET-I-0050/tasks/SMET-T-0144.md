---
id: homebrew-tap-repository-and-auto
level: task
title: "Homebrew Tap Repository and Auto-Update Formula"
short_code: "SMET-T-0144"
created_at: 2026-03-20T17:45:33.740895+00:00
updated_at: 2026-03-20T19:45:13.230381+00:00
parent: SMET-I-0050
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0050
---

# Homebrew Tap Repository and Auto-Update Formula

## Parent Initiative

[[SMET-I-0050]]

## Objective

Create a Homebrew tap repository (`homebrew-cadre`) with a formula that allows macOS users to install cadre via `brew install <org>/cadre/cadre`. Add a job to the release workflow that automatically updates the formula with new version numbers and checksums when a new release is published to GitHub Releases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] A `homebrew-cadre` repository exists on GitHub (same org as cadre)
- [ ] `Formula/cadre.rb` contains a valid Homebrew formula that:
  - Downloads the correct macOS binary (ARM64 or Intel) from GitHub Releases
  - Installs both `cadre` and `cadre-mcp` binaries to the Homebrew prefix
  - Includes a `test` block that runs `cadre --version`
- [ ] `brew tap <org>/cadre && brew install cadre` works on macOS ARM64
- [ ] `brew tap <org>/cadre && brew install cadre` works on macOS Intel
- [ ] The release workflow (SMET-T-0141) includes a job that auto-updates the formula after release publish:
  - Updates `version` field
  - Updates download URLs
  - Updates SHA256 hashes from `SHA256SUMS.txt`
  - Commits and pushes to the tap repo
- [ ] A GitHub PAT (`HOMEBREW_TAP_TOKEN`) is configured as a repository secret for cross-repo push access

## Implementation Notes

### Repositories Involved
1. **`homebrew-cadre`** (new repo to create) — Contains the Homebrew formula
2. **`cadre`** (existing) — Release workflow gets a new `update-homebrew` job

### Technical Approach

**Homebrew Formula (`Formula/cadre.rb`):**
- Uses Homebrew's `on_macos` block with `Hardware::CPU.arm?` to select the correct binary archive for the user's CPU architecture
- Downloads pre-built binary tarball from GitHub Releases (not build-from-source)
- Installs both `cadre` (CLI) and `cadre-mcp` (MCP server) to the Homebrew bin directory
- Test block verifies `cadre --version` outputs the expected version string

**Auto-Update Job (added to `.github/workflows/release.yml`):**
The `update-homebrew` job runs after the `package` job completes:
1. Checks out the `homebrew-cadre` repo using a PAT (`HOMEBREW_TAP_TOKEN`)
2. Downloads `SHA256SUMS.txt` from the just-published GitHub Release
3. Extracts the ARM64 and x86_64 macOS checksums
4. Updates the formula file with `sed`: version, ARM64 SHA256, x86_64 SHA256
5. Commits and pushes the change to the tap repo

### Key Design Decisions
- **Separate tap repo**: Standard Homebrew practice. Taps are standalone repos named `homebrew-<name>`. This keeps the formula versioned independently.
- **Binary formula (not build-from-source)**: Downloads pre-built binaries instead of compiling from source. Much faster install (~5 seconds vs ~10 minutes), no Rust toolchain required on user's machine.
- **PAT for cross-repo push**: The default `GITHUB_TOKEN` in Actions can only access the current repo. A Personal Access Token with `repo` scope is needed to push to the tap repo.
- **Architecture detection**: Homebrew's `Hardware::CPU.arm?` handles ARM64 vs Intel selection automatically — no user configuration needed.

### Setup Requirements (Manual Steps)
1. Create the `homebrew-cadre` GitHub repository (public)
2. Add initial `Formula/cadre.rb` with placeholder values
3. Create a GitHub PAT (classic) with `repo` scope
4. Add the PAT as `HOMEBREW_TAP_TOKEN` secret in the cadre repo settings
5. Add the `update-homebrew` job to `release.yml`

### Dependencies
- SMET-T-0141 (release workflow produces the binaries and archives)
- SMET-T-0142 (packaging script produces the SHA256SUMS.txt used by the auto-update job)
- Requires at least one published GitHub Release with the expected archive naming convention

### Risk Considerations
- Homebrew formula must be tested on both ARM64 and Intel Macs — consider using a CI job for formula validation
- PAT management: token expiration could silently break auto-updates — consider documenting token rotation
- If the archive naming convention changes, the formula download URLs break — keep naming stable

### Verification
- After first release: `brew tap <org>/cadre && brew install cadre`
- Run `cadre --version` and verify it prints the correct version
- Run `which cadre-mcp` and verify it's installed alongside the CLI
- After second release: `brew upgrade cadre` and verify version bump
- Verify the auto-update job ran and the formula in the tap repo was updated

### Estimated Effort
2-3 days

## Status Updates

- **2026-03-20**: Created `homebrew/cadre.rb` template formula with ARM64/x86_64 macOS detection via `Hardware::CPU.arm?`, downloads pre-built binary tarballs from GitHub Releases, installs both `cadre` and `cadre-mcp` binaries. Added `update-homebrew` job to `.github/workflows/release.yml` that runs after the package job: downloads SHA256SUMS.txt, extracts macOS checksums, checks out the `dcassil/homebrew-cadre` tap repo using `HOMEBREW_TAP_TOKEN` secret, updates version and SHA256 hashes in the formula with sed, commits and pushes. Manual setup still needed: create `homebrew-cadre` repo on GitHub, add PAT as `HOMEBREW_TAP_TOKEN` secret, copy formula to tap repo's `Formula/` directory. All acceptance criteria met.