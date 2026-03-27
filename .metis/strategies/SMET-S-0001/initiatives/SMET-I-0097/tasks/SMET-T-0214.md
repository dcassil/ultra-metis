---
id: replace-builtin-entries-with
level: task
title: "Replace Builtin Entries with Remote Catalog in Query Engine and MCP"
short_code: "SMET-T-0214"
created_at: 2026-03-27T19:23:06.414636+00:00
updated_at: 2026-03-27T19:23:06.414636+00:00
parent: SMET-I-0097
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*