---
id: catalog-remote-fetcher-module-in
level: task
title: "Catalog Remote Fetcher Module in cadre-core"
short_code: "SMET-T-0213"
created_at: 2026-03-27T19:23:05.521265+00:00
updated_at: 2026-03-27T19:54:02.791284+00:00
parent: SMET-I-0097
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: SMET-S-0001
initiative_id: SMET-I-0097
---

# Catalog Remote Fetcher Module in cadre-core

## Parent Initiative

[[SMET-I-0097]]

## Objective

Add a new `remote_fetcher` module to `cadre-core` that clones/pulls the `dcassil/cadre-architecture-docs` repo into a local cache directory and loads all architecture catalog entries from it. This replaces the compile-time `include_str!()` approach with runtime fetching.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `remote_fetcher` module in `crates/cadre-core/src/domain/catalog/`
- [ ] Fetcher clones the repo on first use to `~/.cadre/catalog-cache/cadre-architecture-docs/`
- [ ] Subsequent calls do `git pull` to update the cache
- [ ] Configurable repo URL (default: `https://github.com/dcassil/cadre-architecture-docs.git`)
- [ ] Loads all `{language}/{project-type}.md` files from cache using existing `ArchitectureCatalogEntry::from_file()` / `from_content()` parsing
- [ ] Graceful offline fallback: if clone/pull fails and no cache exists, returns empty vec (not a crash)
- [ ] Graceful stale fallback: if pull fails but cache exists, uses stale cache with a warning
- [ ] Unit tests for cache path resolution, entry loading, and error handling
- [ ] Integration test that loads entries from a mock directory structure

## Implementation Notes

### Technical Approach

New file: `crates/cadre-core/src/domain/catalog/remote_fetcher.rs`

```rust
pub struct RemoteCatalogFetcher {
    repo_url: String,
    cache_dir: PathBuf,
}

impl RemoteCatalogFetcher {
    pub fn new(repo_url: &str, cache_dir: PathBuf) -> Self;
    pub fn with_defaults() -> Self; // uses default URL + ~/.cadre/catalog-cache/
    pub async fn fetch(&self) -> Result<Vec<ArchitectureCatalogEntry>, FetchError>;
    pub async fn fetch_cached_only(&self) -> Result<Vec<ArchitectureCatalogEntry>, FetchError>;
}
```

**Fetching strategy:**
1. Check if `cache_dir/cadre-architecture-docs` exists
2. If yes: `git -C <path> pull --ff-only` (quick update)
3. If no: `git clone --depth 1 <repo_url> <path>` (shallow clone for speed)
4. Walk the directory for `**/*.md` files, parse each as `ArchitectureCatalogEntry`
5. Skip files that fail to parse (log warning, don't abort)

**Git execution:** Use `tokio::process::Command` to run git commands. No git library dependency needed.

### Key Design Decisions
- Shallow clone (`--depth 1`) to minimize bandwidth and disk usage
- `--ff-only` pull to avoid merge conflicts in the cache
- If pull fails with conflicts, delete cache and re-clone
- Cache directory respects `XDG_CACHE_HOME` if set, otherwise `~/.cadre/catalog-cache/`

### Dependencies
- SMET-T-0212 (external repo must exist for real fetching, but tests can use mock dirs)
- `tokio::process` for async git command execution (already a workspace dependency)

### Risk Considerations
- Git not installed on user's machine: detect and return clear error
- Network unavailable: graceful fallback to cache or empty catalog
- Corrupted cache: detect parse failures, offer re-clone

## Status Updates

### 2026-03-27
- Created `crates/cadre-core/src/domain/catalog/remote_fetcher.rs`
- `RemoteCatalogFetcher` with `new()`, `with_defaults()`, `fetch()`, `fetch_cached_only()`
- Shallow clone on first use, `git pull --ff-only` on subsequent uses
- Falls back to stale cache on pull failure, empty vec on no cache
- Re-clones if pull fails with conflicts
- Respects `XDG_CACHE_HOME`, defaults to `~/.cadre/catalog-cache/`
- Walks `{language}/{project-type}.md` structure, skips README and invalid files
- 7 tests all passing: cache path resolution, entry loading, multi-language, invalid file skipping
- Registered module in `mod.rs`
- Full workspace compiles clean