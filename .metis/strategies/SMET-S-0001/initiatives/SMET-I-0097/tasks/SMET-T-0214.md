---
id: replace-builtin-entries-with
level: task
title: "Replace Builtin Entries with Remote Catalog in Query Engine and MCP"
short_code: "SMET-T-0214"
created_at: 2026-03-27T19:23:06.414636+00:00
updated_at: 2026-03-27T20:09:36.644492+00:00
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

# Replace Builtin Entries with Remote Catalog in Query Engine and MCP

## Parent Initiative

[[SMET-I-0097]]

## Objective

Wire the remote fetcher into the `CatalogQueryEngine` and MCP `query_architecture_catalog` tool so entries come from the external repo at runtime. Remove the hardcoded `builtin_entries.rs` and `builtin_data/` directory from the binary.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CatalogQueryEngine` has a new constructor `with_remote()` that uses `RemoteCatalogFetcher`
- [ ] `CatalogQueryEngine::with_remote_and_custom()` merges remote + local custom entries
- [ ] `build_engine_with_custom()` in `custom_loader.rs` updated to use remote fetcher instead of `builtin_entries()`
- [ ] MCP `query_architecture_catalog` tool calls the updated engine (triggers fetch on first query)
- [ ] `builtin_entries.rs` removed (or reduced to an empty `builtin_entries() -> Vec` for backwards compat)
- [ ] `builtin_data/` directory contents removed from the crate (no more `include_str!()`)
- [ ] All existing tests updated or replaced to work with the new flow
- [ ] `make test` passes with all changes
- [ ] `make build` produces a binary that no longer embeds catalog markdown

## Implementation Notes

### Technical Approach

1. **Update `CatalogQueryEngine`** (`query_engine.rs`):
   - Add `async fn with_remote() -> Self` — creates fetcher, loads entries
   - Add `async fn with_remote_and_custom(custom: Vec<...>) -> Self`
   - Keep `with_builtins()` as deprecated/empty for transition if needed

2. **Update `custom_loader.rs`**:
   - `build_engine_with_custom()` should now call `RemoteCatalogFetcher::with_defaults().fetch()` instead of `builtin_entries()`
   - Merge: remote entries + custom `.cadre/catalog/` entries

3. **Update MCP tool** (`crates/cadre-mcp/src/tools/query_architecture_catalog.rs`):
   - Currently uses `CatalogQueryEngine::with_builtins()` — switch to `build_engine_with_custom(metis_path)`
   - The engine construction is now async (fetcher), so adjust call sites

4. **Remove builtin data**:
   - Delete `builtin_data/*.md` files
   - Gut `builtin_entries.rs` — either remove entirely or leave as `pub fn builtin_entries() -> Vec<...> { vec![] }`
   - Remove `include_str!()` macros

5. **Update tests**:
   - Tests that relied on `with_builtins()` returning 5 entries need updating
   - Use mock directories with test entries for deterministic testing
   - Keep integration tests that verify the full query pipeline

### Files to Modify
- `crates/cadre-core/src/domain/catalog/query_engine.rs`
- `crates/cadre-core/src/domain/catalog/custom_loader.rs`
- `crates/cadre-core/src/domain/catalog/builtin_entries.rs` (remove/gut)
- `crates/cadre-core/src/domain/catalog/mod.rs` (update module exports)
- `crates/cadre-mcp/src/tools/query_architecture_catalog.rs`
- All test files referencing `builtin_entries` or `with_builtins()`

### Dependencies
- SMET-T-0213 (remote fetcher module must exist)
- SMET-T-0212 (external repo should exist for real integration testing)

### Risk Considerations
- Breaking change for anyone using `with_builtins()` — provide empty fallback
- Async constructor pattern may require refactoring some call sites
- Tests that expect exactly 5 entries will break — must update

## Status Updates

### 2026-03-27
- **query_engine.rs**: Added `with_remote()` and `with_remote_and_custom()` async constructors
- **builtin_entries.rs**: `builtin_entries()` now returns empty vec; added `test_builtin_entries()` gated behind `cfg(any(test, feature = "test-utils"))` with all 5 JS entries preserved for tests
- **builtin_data/**: Deleted all 9 markdown files and directory — no more `include_str!()`
- **custom_loader.rs**: `build_engine_with_custom()` now uses `RemoteCatalogFetcher::with_defaults().fetch()` + local custom entries
- **MCP tools**: All 3 tools (`query_architecture_catalog`, `evaluate_brownfield`, `list_catalog_languages`) switched from `with_builtins()` to `with_remote().await`
- **lib.rs**: Added `RemoteCatalogFetcher`, `FetchError`, `DEFAULT_REPO_URL`, `default_cache_dir` to public exports
- **All tests updated**: ~30+ test references switched from `builtin_entries()` to `test_builtin_entries()` across query_engine, selection_flow, evaluator, pattern_matcher, and integration tests
- **Cargo.toml**: Added `cadre-core = { path = ".", features = ["test-utils"] }` to dev-dependencies so integration tests can access test entries
- Full workspace: 0 failures, 804+ tests pass, release build succeeds