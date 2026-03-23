---
id: cross-platform-release-build
level: task
title: "Cross-Platform Release Build Workflow with GitHub Actions"
short_code: "SMET-T-0141"
created_at: 2026-03-20T17:45:31.006831+00:00
updated_at: 2026-03-20T19:44:00.884826+00:00
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

# Cross-Platform Release Build Workflow with GitHub Actions

## Parent Initiative

[[SMET-I-0050]]

## Objective

Create a GitHub Actions workflow (`.github/workflows/release.yml`) that triggers when release-please creates a git tag, builds both `cadre` (CLI) and `cadre-mcp` (MCP server) binaries for all 5 target platforms, packages them into distributable archives, and uploads them as GitHub Release assets. This is the core build pipeline that produces all distributable artifacts.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.github/workflows/release.yml` exists and triggers on `v*.*.*` tags
- [ ] Build matrix covers all 5 target triples:
  - `aarch64-apple-darwin` (macOS ARM64)
  - `x86_64-apple-darwin` (macOS Intel)
  - `x86_64-unknown-linux-gnu` (Linux x86_64)
  - `aarch64-unknown-linux-gnu` (Linux ARM64)
  - `x86_64-pc-windows-msvc` (Windows x64)
- [ ] Both `cadre` and `cadre-mcp` binaries are built for each platform
- [ ] macOS cross-compilation between ARM64 and x86_64 works correctly
- [ ] Linux ARM64 uses `cross` for cross-compilation from x86_64 runner
- [ ] Rust caching via `Swatinem/rust-cache@v2` is configured with per-target cache keys
- [ ] Built binaries are uploaded as workflow artifacts for the packaging step
- [ ] Packaging job assembles distributable archives using `scripts/package.sh` (SMET-T-0142)
- [ ] Archives and SHA256 checksums are uploaded to the GitHub Release via `softprops/action-gh-release@v2`

## Implementation Notes

### File to Create
`.github/workflows/release.yml`

### Technical Approach

**Two-phase pipeline:**

**Phase 1 — Build Matrix (parallel across 5 platforms):**
Each matrix entry runs on the appropriate runner and builds both binaries:

| Target Triple | Runner | Cross-compile? | Build Tool |
|---------------|--------|----------------|------------|
| `aarch64-apple-darwin` | `macos-latest` | No (native ARM64) | `cargo` |
| `x86_64-apple-darwin` | `macos-latest` | Yes (ARM64 host -> x86_64 target) | `cargo` |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | No (native x86_64) | `cargo` |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04` | Yes (x86_64 host -> ARM64 target via Docker) | `cross` |
| `x86_64-pc-windows-msvc` | `windows-latest` | No (native x86_64) | `cargo` |

Each build job:
1. Checks out code (`actions/checkout@v4`)
2. Installs Rust stable with target (`dtolnay/rust-toolchain@stable`)
3. Configures Rust cache (`Swatinem/rust-cache@v2` with `key: release-${{ matrix.target }}`)
4. Installs `cross` if needed (Linux ARM64 only)
5. Builds both `-p cadre-cli` and `-p cadre-mcp` with `--release --target`
6. Uploads raw binaries as workflow artifacts (`actions/upload-artifact@v4`)

**Phase 2 — Package and Release (single job, runs after all builds):**
1. Downloads all build artifacts (`actions/download-artifact@v4`)
2. Runs `scripts/package.sh ${{ github.ref_name }}` to create archives + checksums
3. Uploads everything in `dist/` to the GitHub Release (`softprops/action-gh-release@v2`)

### Key Design Decisions
- **`fail-fast: false`**: Build all platforms even if one fails for better debugging
- **macOS cross-compilation**: `macos-latest` is ARM64, so `aarch64-apple-darwin` is native. For `x86_64-apple-darwin`, Rust's built-in cross-compilation works without external tools.
- **Linux ARM64 via `cross`**: The `cross` tool uses Docker to provide the ARM64 toolchain on an x86_64 runner — simpler than finding a native ARM64 Linux runner
- **`softprops/action-gh-release`**: Well-maintained, supports file globs, and `generate_release_notes: true` provides auto-generated notes

### Binary Names by Platform
| Target | CLI Binary | MCP Binary |
|--------|-----------|------------|
| macOS/Linux | `cadre` | `cadre-mcp` |
| Windows | `cadre.exe` | `cadre-mcp.exe` |

### Dependencies
- SMET-T-0140 (release-please creates the tags that trigger this workflow)
- SMET-T-0142 (packaging script used by the package job)

### Risk Considerations
- macOS x86_64 cross-compilation from ARM64 runner may have issues with native dependencies — test early
- `cross` installation adds time to Linux ARM64 builds — consider caching the `cross` binary
- Windows builds may need special handling for binary extensions (`.exe`)

### Verification
- Create a test tag (`v0.0.0-test`) to trigger the workflow without affecting real releases
- Verify all 5 platform builds succeed in the Actions UI
- Verify artifacts are correctly uploaded to the GitHub Release
- Download binaries on available platforms and smoke-test: `./cadre --version`

### Estimated Effort
3-5 days

## Status Updates

- **2026-03-20**: Created `.github/workflows/release.yml` with two-phase pipeline. Phase 1: build matrix across all 5 targets (aarch64-apple-darwin native on macos-latest, x86_64-apple-darwin cross-compiled on macos-latest, x86_64-unknown-linux-gnu native on ubuntu-22.04, aarch64-unknown-linux-gnu via cross on ubuntu-22.04, x86_64-pc-windows-msvc native on windows-latest). Uses `fail-fast: false`, per-target cache keys, and separate upload steps for unix/windows binary names. Phase 2: downloads all artifacts, runs `scripts/package.sh` with the tag name, uploads archives + checksums to GitHub Release via `softprops/action-gh-release@v2`. All acceptance criteria met.