# Benchmark Report: Cadre vs Original Metis

**Date**: 2026-03-17
**Environment**: macOS Darwin 24.4.0
**Cadre**: Release binary (Rust CLI)
**Original Metis**: MCP plugin (TypeScript, JSON-RPC over stdio)

## Executive Summary

Ultra-metis is **~200x faster** than original metis for individual operations due to the elimination of MCP protocol overhead. However, original metis has **better validation**, **richer templates**, and **more robust error handling**. The speed advantage of cadre is dramatic and real, but correctness gaps need to be addressed before it can replace the original.

## Methodology

Both systems were given identical tasks using fresh project directories (`/tmp/bench-cadre` and `/tmp/bench-metis-original`). Ultra-metis was invoked directly as a CLI binary; original metis was invoked via MCP tool calls from within a Claude Code session. Timestamps were captured using Python `time.time()` with millisecond precision.

**Important caveat**: The MCP timing includes the full round-trip through Claude Code's tool-calling infrastructure (JSON-RPC serialization, stdio transport, response parsing). This overhead is inherent to MCP-based tools and is the actual latency experienced by users/agents. Ultra-metis CLI times are pure execution times.

## Results

### Scenario 1: Project Bootstrap

| Metric | Cadre | Original Metis | Winner |
|--------|-------------|----------------|--------|
| Time | **38ms** | 7,739ms | Cadre (204x) |
| Output size | 75 bytes | 393 bytes | Cadre (leaner) |
| Pass/Fail | PASS | PASS | Tie |

**Notes**: Ultra-metis creates a `.cadre/` directory with config. Original metis creates `.metis/` with SQLite DB, vision template, and config. Both succeed, but metis does more setup work.

### Scenario 2: Planning Workflow (Vision + Initiative + 3 Tasks)

| Operation | Cadre | Original Metis |
|-----------|-------------|----------------|
| Create Vision | 44ms | N/A (auto-created) |
| Create Initiative | 33ms | 8,621ms |
| Create Task 1 | 37ms | 8,015ms |
| Create Task 2 | 30ms | 8,763ms |
| Create Task 3 | 37ms | 8,169ms |
| **Total** | **181ms** | **33,568ms** |

| Metric | Cadre | Original Metis | Winner |
|--------|-------------|----------------|--------|
| Total time | **181ms** | 33,568ms | Cadre (185x) |
| Total output | 322 bytes | 1,520 bytes | Cadre (leaner) |
| Multiple visions | Allowed | Only 1 allowed | Depends on use case |

**Notes**: Original metis only allows one vision per project (enforced constraint). Ultra-metis allows multiple visions. Both create proper document hierarchies.

### Scenario 3: Phase Transitions (5 transitions: discovery through completed)

| Transition | Cadre | Original Metis |
|-----------|-------------|----------------|
| discovery -> design | 42ms | 11,787ms |
| design -> ready | 35ms | 13,337ms |
| ready -> decompose | 40ms | 7,293ms |
| decompose -> active | 36ms | 7,906ms |
| active -> completed | 42ms | 8,790ms |
| **Total** | **195ms** | **49,113ms** |

| Metric | Cadre | Original Metis | Winner |
|--------|-------------|----------------|--------|
| Total time | **195ms** | 49,113ms | Cadre (252x) |
| Avg per transition | **39ms** | 9,823ms | Cadre |
| Consistency | 35-42ms range | 7-13s range | Cadre |

### Scenario 4: Search and Query (12 documents)

| Operation | Cadre | Original Metis |
|-----------|-------------|----------------|
| Search "database" | 43ms (2 results) | 7,831ms (2 results) |
| Search "API" | 36ms (2 results) | 10,901ms (9 results) |
| List all | 35ms (12 results) | 11,472ms (12 results) |

| Metric | Cadre | Original Metis | Winner |
|--------|-------------|----------------|--------|
| Avg search time | **38ms** | 10,068ms | Cadre (265x) |
| Search precision | Title-only match | Full-text (title+body) | Context-dependent |
| Search "API" accuracy | 2 results (correct) | 9 results (false positives) | Cadre |

**Notes**: Original metis searches document body content, which returned 9 results for "API" when only 2 documents have "API" in their title. The body templates contain "API" text in placeholder sections. Ultra-metis title-only search was more precise here, but full-text search could be valuable when documents have real content.

### Scenario 5: Error Handling

| Error Case | Cadre | Original Metis |
|-----------|-------------|----------------|
| Read non-existent doc | Error returned (score: 3/5) | Error returned (score: 4/5) |
| Create with bad parent | **No error - BUG** (score: 1/5) | Error returned (score: 5/5) |
| Transition past completed | **Silent no-op - BUG** (score: 2/5) | Error returned (score: 4/5) |
| Invalid short code | Error returned (score: 3/5) | Error returned (score: 4/5) |

| Metric | Cadre | Original Metis | Winner |
|--------|-------------|----------------|--------|
| Avg error quality | 2.25/5 | **4.25/5** | Original Metis |
| Errors caught | 2/4 | **4/4** | Original Metis |
| Avg error time | 41ms | 9,287ms | Cadre |

**Critical findings for Cadre**:
1. **BUG**: Allows creating tasks with non-existent parent IDs (should validate parent exists)
2. **BUG**: Transitioning a completed initiative silently reports `completed -> completed` instead of erroring

### Scenario 6: Document Read and Template Quality

| Metric | Cadre | Original Metis | Winner |
|--------|-------------|----------------|--------|
| Read time | **39ms** | 10,218ms | Cadre (262x) |
| Output size | 927 bytes | 3,200 bytes | Context-dependent |
| Template quality | 3/5 | **5/5** | Original Metis |

**Notes**: Original metis templates are significantly richer with conditional sections (`[REQUIRED]`, `[CONDITIONAL]`), detailed guidance for different initiative types (requirements-heavy, user-facing, technically complex), and structured sub-sections for testing strategy, UI/UX, and architecture. Ultra-metis templates are minimal with basic placeholder text.

## Aggregate Summary

### Speed Comparison

| Scenario | Cadre | Original Metis | Speedup |
|----------|-------------|----------------|---------|
| Init | 38ms | 7,739ms | **204x** |
| Planning (5 ops) | 181ms | 33,568ms | **185x** |
| Transitions (5 ops) | 195ms | 49,113ms | **252x** |
| Search/Query (3 ops) | 114ms | 30,204ms | **265x** |
| Error handling (4 ops) | 163ms | 37,148ms | **228x** |
| Read document | 39ms | 10,218ms | **262x** |
| **Total** | **730ms** | **167,990ms** | **230x** |

### Quality Comparison

| Dimension | Cadre | Original Metis | Winner |
|-----------|-------------|----------------|--------|
| Speed | 5/5 | 1/5 | Cadre |
| Template richness | 3/5 | 5/5 | Original Metis |
| Error handling | 2/5 | 4/5 | Original Metis |
| Validation correctness | 2/5 | 5/5 | Original Metis |
| Search precision | 4/5 | 3/5 | Cadre |
| Output conciseness | 5/5 | 3/5 | Cadre |

### Token Usage Implications

For AI agent workflows, output size directly impacts token consumption:

| Workflow | Cadre Tokens (est.) | Original Metis Tokens (est.) | Savings |
|----------|---------------------------|------------------------------|---------|
| Init | ~20 | ~100 | 80% |
| Create doc | ~18 | ~100 | 82% |
| Transition | ~10 | ~50 | 80% |
| Read initiative | ~230 | ~800 | 71% |
| Search (3 ops) | ~460 | ~525 | 12% |

Ultra-metis produces significantly less output per operation, which translates to ~70-80% token savings for most operations. This compounds across a full workflow with dozens of operations.

## Bugs Found in Cadre

| # | Severity | Description | Expected Behavior |
|---|----------|-------------|-------------------|
| 1 | **High** | Creating task with non-existent parent succeeds | Should return error: "Parent document not found" |
| 2 | **Medium** | Transitioning completed initiative reports `completed -> completed` | Should return error: "No valid transition from completed" |

## Recommendations

### Cadre Strengths (keep/enhance)
1. **Speed is the killer feature** - 230x faster is transformative for AI agent workflows
2. **Concise output** - less token waste, cleaner agent interactions
3. **Precise search** - title matching avoids template noise in results

### Cadre Gaps (fix before shipping)
1. **Parent validation** - must validate parent document exists before creating children
2. **Terminal phase handling** - must reject transitions from terminal phases
3. **Template quality** - initiative templates need conditional sections and richer guidance
4. **Error messages** - add recovery suggestions (e.g., "Did you mean BENCH-I-0002?")
5. **Single vision constraint** - consider whether multiple visions should be allowed (original metis enforces one)

### Overall Assessment

Ultra-metis delivers on its core promise: dramatically faster operations with lower token overhead. The speed difference is not marginal -- it is **two orders of magnitude**. For an AI agent making 50+ document operations per session, this means the difference between ~8 minutes of cumulative MCP latency vs ~2 seconds.

However, the validation and error handling gaps are real. An agent that can create orphan documents or silently fail on invalid transitions will produce corrupted project state. These must be fixed before cadre can reliably replace original metis in production workflows.

**Bottom line**: Ultra-metis is faster and leaner. Original metis is more correct and feature-rich. The path forward is clear: fix the validation bugs, enrich the templates, and cadre will be strictly better.
