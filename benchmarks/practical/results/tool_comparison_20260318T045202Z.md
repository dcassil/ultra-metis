# Tool Comparison Report: ultra-metis vs original-metis

**Date**: 2026-03-18 04:52:02 UTC  
**Scenario**: File Processing Toolkit (3 modules)

## Executive Summary

- Both tools produce similarly complete documents (delta < 5%)
- Completeness delta: **+0.0%** (ultra-metis − original-metis)
- Placeholder reduction: **-3.3** placeholders/doc (original − ultra)

## Per-Tool Results

| Metric | ultra-metis | original-metis | Delta |
|--------|-------------|----------------|-------|
| Templates tested | 3 | 3 | — |
| Avg completeness | 77.8% | 77.8% | +0.0% |
| Avg placeholder count | 8.3 | 5.0 | +3.3 |
| Total filled sections | 9 | 9 | +0 |
| Total empty sections | 3 | 3 | +0 |
| Tokens used | 14922 | 18482 | -3560 |
| Time (s) | 144.0 | 177.5 | — |

## Interpretation

- Templates perform similarly — AI fills both equally well
- Template structure may not be the dominant factor in output quality
- ultra-metis templates leave 3.3 more unfilled placeholders on average
