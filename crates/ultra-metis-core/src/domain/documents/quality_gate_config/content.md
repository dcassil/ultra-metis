# {{ title }}

## Default Gates

<!-- Quality gate thresholds that apply to all transitions unless overridden. -->
| Metric | Threshold Type | Value | Severity | Description |
|--------|---------------|-------|----------|-------------|
| (metric name) | (absolute / relative / trend) | (value) | (blocking / advisory) | (what this gate checks) |

## Transition Overrides

<!-- Stricter or relaxed gates for specific transitions. These override the defaults above. -->

### active -> completed
<!-- Typically the strictest gates. All quality checks must pass. -->
| Metric | Threshold Type | Value | Severity |
|--------|---------------|-------|----------|
| (metric) | (type) | (value) | (severity) |

### ready -> active
<!-- Entry gates. May be more relaxed to allow work to begin. -->
| Metric | Threshold Type | Value | Severity |
|--------|---------------|-------|----------|
| (metric) | (type) | (value) | (severity) |

## Notes

<!-- Context about threshold choices, calibration history, or known limitations. -->
- **Calibration Date**: (when thresholds were last reviewed)
- **Rationale**: (why these specific thresholds were chosen)
- **Known Gaps**: (metrics not yet covered that should be added)