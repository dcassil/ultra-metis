# {{ title }}

## Metrics

<!-- Define every quality metric being tracked in this baseline. -->
| Metric | Definition | Tool | Unit |
|--------|-----------|------|------|
| (metric name) | (what it measures) | (tool that captures it) | (unit of measurement) |

## Thresholds

<!-- Acceptable ranges for each metric. Define pass, warn, and fail boundaries. -->
| Metric | Pass | Warn | Fail |
|--------|------|------|------|
| (metric) | (>= value) | (>= value) | (< value) |

## Baseline Values

<!-- The actual measured values at baseline capture time. -->
| Metric | Value | Captured At | Confidence |
|--------|-------|------------|------------|
| (metric) | (value) | (timestamp) | (high/medium/low) |

## Configuration

<!-- How these metrics are captured. Reference the tooling setup. -->
- **RulesConfig**: (short code of the RulesConfig that defines analysis rules)
- **Tool Versions**: (specific tool versions used for baseline capture)
- **Capture Command**: (the exact command or process used to produce these numbers)

## Notes

<!-- Context about this baseline snapshot. -->
- **Capture Trigger**: (what caused this baseline to be taken -- initial setup / architecture change / remediation completion)
- **Known Limitations**: (metrics not yet captured, tools not yet configured)
- **Next Review**: (when this baseline should be refreshed)