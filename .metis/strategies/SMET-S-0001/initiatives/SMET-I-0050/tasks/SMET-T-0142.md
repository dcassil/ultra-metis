---
id: release-artifact-packaging-script
level: task
title: "Release Artifact Packaging Script and Checksum Generation"
short_code: "SMET-T-0142"
created_at: 2026-03-20T17:45:31.896503+00:00
updated_at: 2026-03-20T19:42:45.162927+00:00
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

# Release Artifact Packaging Script and Checksum Generation

## Parent Initiative

[[SMET-I-0050]]

## Objective

Create a `scripts/package.sh` script that takes the raw binaries produced by the cross-platform build matrix and packages them into distributable archives with consistent naming, directory structure, and SHA256 checksums. This script is called by the release workflow's packaging job (SMET-T-0141) and can also be run locally via `make package` (SMET-T-0143).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `scripts/package.sh` exists and is executable (`chmod +x`)
- [ ] Script accepts a version tag (e.g., `v0.2.0`) as its first argument
- [ ] For each platform's binaries, creates a distributable archive:
  - `.tar.gz` for macOS and Linux targets
  - `.zip` for Windows targets
- [ ] Archive naming follows: `cadre-{version}-{target-triple}.{ext}`
- [ ] Each archive contains a top-level directory with: both binaries, LICENSE, README.md
- [ ] A `SHA256SUMS.txt` file is generated containing checksums for all archives
- [ ] All output goes to a `dist/` directory (created if it doesn't exist)
- [ ] Script fails with a clear error message if expected binary artifacts are missing
- [ ] Script uses `set -euo pipefail` for strict error handling
- [ ] Checksum generation works on both Linux (`sha256sum`) and macOS (`shasum -a 256`)

## Implementation Notes

### File to Create
`scripts/package.sh`

### Technical Approach

The script iterates over a fixed list of 5 target triples. For each target:

1. **Locate binaries**: Look in `binaries-{target}/` directory (where GitHub Actions `download-artifact` places them). Handle Windows `.exe` suffix.
2. **Create staging directory**: `dist/cadre-{version}-{target}/` containing both binaries + LICENSE + README.md
3. **Set permissions**: `chmod +x` on macOS/Linux binaries
4. **Create archive**: `tar -czf` for macOS/Linux, `zip -r` for Windows
5. **Clean staging**: Remove the staging directory after archiving

After all archives are created, generate `SHA256SUMS.txt` with checksums for all archives. Use `sha256sum` (Linux) with fallback to `shasum -a 256` (macOS) for portability.

### Output Structure
```
dist/
  cadre-v0.2.0-aarch64-apple-darwin.tar.gz
  cadre-v0.2.0-x86_64-apple-darwin.tar.gz
  cadre-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
  cadre-v0.2.0-aarch64-unknown-linux-gnu.tar.gz
  cadre-v0.2.0-x86_64-pc-windows-msvc.zip
  SHA256SUMS.txt
```

### Key Design Decisions
- **`set -euo pipefail`**: Fail fast on any error — packaging must be reliable
- **Staging directory approach**: Creates archives with a top-level named directory (not loose files at the root)
- **Portable checksum generation**: Try `sha256sum` first, fall back to `shasum -a 256` for macOS compatibility
- **LICENSE/README optional**: Use `|| true` to avoid failure if these files don't exist yet (they should, but the script shouldn't break the release)

### Dependencies
- Used by SMET-T-0141 (release build workflow calls this in the packaging job)
- Used by SMET-T-0143 (`make package` target invokes this script)

### Verification
- Run locally after `make build`: `bash scripts/package.sh v0.0.0-test` (will package for current platform only in local mode)
- Verify archive contents: `tar -tzf dist/cadre-v0.0.0-test-aarch64-apple-darwin.tar.gz`
- Verify checksums: `cd dist && shasum -a 256 -c SHA256SUMS.txt`
- Verify the script fails gracefully when binaries are missing (clear error message, non-zero exit)

### Estimated Effort
1-2 days

## Status Updates

- **2026-03-20**: Created `scripts/package.sh` with `set -euo pipefail`. Handles all 5 target triples, creates tar.gz for macOS/Linux and zip for Windows. Falls back to `target/release/` for local builds (auto-detects current platform). Generates SHA256SUMS.txt with portable sha256sum/shasum fallback. Archives include top-level named directory with binaries, LICENSE, and README.md. Script is executable. All acceptance criteria met.