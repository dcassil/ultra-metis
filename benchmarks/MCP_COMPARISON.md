# MCP Benchmark: Cadre vs Original Metis (Apples-to-Apples)

**Date**: 2026-03-17 23:04:56
**Transport**: Both servers via stdio MCP (JSON-RPC 2.0, newline-delimited)
**Cadre**: Rust MCP server (server startup: 883ms)
**Original Metis**: TypeScript MCP server v1.1.0 (server startup: 224ms)

## Executive Summary

Across **18 operations** over identical MCP stdio transport, cadre completed in **10ms** vs original metis in **128ms** — **12.7x faster**.

Both servers communicate via the same newline-delimited JSON-RPC 2.0 over stdio. The previous benchmark (REPORT.md) compared cadre CLI vs original metis MCP, giving cadre an unfair transport advantage (~200x). This benchmark isolates actual server performance by using identical transport for both.

## Scenario: Project Bootstrap

| Operation | Cadre (ms) | Original Metis (ms) | Speedup | Ultra OK | Orig OK |
|-----------|:----------------:|:-------------------:|:-------:|:--------:|:-------:|
| init | 1.8 | 14.2 | 8.1x | Y | Y |
| **Total** | **1.8** | **14.2** | **8.1x** | | |

## Scenario: Planning Workflow

| Operation | Cadre (ms) | Original Metis (ms) | Speedup | Ultra OK | Orig OK |
|-----------|:----------------:|:-------------------:|:-------:|:--------:|:-------:|
| create_vision | 2.2 | 0.0 | - | Y | Y |
| vision_to_review | 0.6 | 9.5 | 14.8x | Y | Y |
| vision_to_published | 0.4 | 8.7 | 21.2x | Y | Y |
| create_initiative | 1.2 | 8.0 | 6.5x | Y | Y |
| init_to_design | 0.4 | 7.8 | 21.5x | Y | Y |
| init_to_ready | 0.3 | 8.0 | 29.3x | Y | Y |
| init_to_decompose | 0.3 | 7.5 | 27.6x | Y | Y |
| create_task_1 | 0.7 | 7.4 | 10.4x | Y | Y |
| create_task_2 | 0.6 | 7.8 | 12.0x | Y | Y |
| create_task_3 | 0.6 | 7.4 | 11.9x | Y | Y |
| **Total** | **7.4** | **72.2** | **9.8x** | | |

## Scenario: Search and Query

| Operation | Cadre (ms) | Original Metis (ms) | Speedup | Ultra OK | Orig OK |
|-----------|:----------------:|:-------------------:|:-------:|:--------:|:-------:|
| search_parser | 0.3 | 6.2 | 20.3x | Y | Y |
| list_all | 0.3 | 5.6 | 19.2x | Y | Y |
| read_initiative | 0.1 | 6.0 | 109.9x | Y | Y |
| **Total** | **0.7** | **17.8** | **27.3x** | | |

## Scenario: Document Edit

| Operation | Cadre (ms) | Original Metis (ms) | Speedup | Ultra OK | Orig OK |
|-----------|:----------------:|:-------------------:|:-------:|:--------:|:-------:|
| edit_document | 0.2 | 5.5 | 30.1x | Y | Y |
| **Total** | **0.2** | **5.5** | **30.1x** | | |

## Scenario: Error Handling

| Operation | Cadre (ms) | Original Metis (ms) | Speedup | Ultra OK | Orig OK |
|-----------|:----------------:|:-------------------:|:-------:|:--------:|:-------:|
| read_nonexistent | 0.0 | 7.4 | 211.9x | Y | Y |
| create_bad_parent | 0.1 | 5.9 | 91.7x | Y | Y |
| transition_invalid | 0.0 | 5.4 | 323.6x | Y | Y |
| **Total** | **0.1** | **18.7** | **161.1x** | | |

## Aggregate Speed Summary

| Scenario | Cadre (ms) | Original Metis (ms) | Speedup |
|----------|:----------------:|:-------------------:|:-------:|
| Project Bootstrap | 1.8 | 14.2 | 8.1x |
| Planning Workflow | 7.4 | 72.2 | 9.8x |
| Search and Query | 0.7 | 17.8 | 27.3x |
| Document Edit | 0.2 | 5.5 | 30.1x |
| Error Handling | 0.1 | 18.7 | 161.1x |
| **Total (18 ops)** | **10.1** | **128.4** | **12.7x** |

## Output Size Comparison (Token Cost Proxy)

| Scenario | Cadre (bytes) | Original Metis (bytes) | Ratio |
|----------|:-------------------:|:----------------------:|:-----:|
| Project Bootstrap | 110 | 781 | 7.1x |
| Planning Workflow | 1087 | 2155 | 2.0x |
| Search and Query | 3892 | 4353 | 1.1x |
| Document Edit | 42 | 85 | 2.0x |
| Error Handling | 271 | 210 | 0.8x |

## Error Handling Details

| Test Case | Cadre | Original Metis |
|-----------|:-----------:|:--------------:|
| read_nonexistent | CAUGHT | CAUGHT |
| create_bad_parent | CAUGHT | CAUGHT |
| transition_invalid | CAUGHT | CAUGHT |

## Scenario: Template Quality (AI Fill-In)

Each tool's initiative template was read via MCP `read_document`, then sent to 
Claude Haiku to fill in for 3 module specs. Results measure how well the template 
guides AI toward complete, placeholder-free content.

| Metric | Cadre | Original Metis | Delta |
|--------|:-----------:|:--------------:|:-----:|
| Template size (chars) | 3400 | 3706 | +306 |
| Avg completeness | 85% | 55% | +31% |
| Avg placeholders/doc | 11.3 | 1.0 | +10.3 |
| Total filled sections | 23 | 14 | +9 |
| Total empty sections | 4 | 3 | +1 |
| Total tokens used | 9907 | 8323 | +1584 |

### Per-Module Breakdown

| Module | Ultra Complete | Orig Complete | Ultra Placeholders | Orig Placeholders |
|--------|:-------------:|:------------:|:------------------:|:-----------------:|
| CSV Parser Module | 89% | 0% | 7 | 0 |
| JSON Transformer | 89% | 89% | 19 | 0 |
| Output Formatter | 78% | 75% | 8 | 3 |

### Template Quality Verdict

**Ultra-metis** templates yield 31% higher completeness scores.

## Methodology

- Both servers spawned as stdio subprocesses, same newline-delimited JSON-RPC 2.0 transport
- Timing: `time.perf_counter()` around each `tools/call` request-response cycle
- Fresh temp directories, no cached state
- Short codes parsed from actual responses (not hardcoded)

**Key difference from REPORT.md**: Previous benchmark compared cadre CLI (direct binary) vs original metis MCP (via Claude Code tool infrastructure). That gave cadre ~200x advantage from transport alone. This benchmark eliminates transport as a variable.

**Template quality**: Claude Haiku fills initiative templates for 3 module specs. Scored on section completeness and remaining placeholder count.
