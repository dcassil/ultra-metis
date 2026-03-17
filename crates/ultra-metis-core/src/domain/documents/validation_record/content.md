# {{ title }}

## Validation Details

<!-- What validation was performed and why. Reference the triggering document by short code. -->
- **Validation Type**: (pre-transition check / quality gate / policy compliance / manual review)
- **Triggered By**: (short code of the document whose transition triggered this validation)
- **Transition**: (from_phase -> to_phase)

## Inputs

<!-- List every artifact, configuration, and data source consumed by this validation run. -->
- **Source Document**: (short code and title)
- **Quality Gate Config**: (short code, if applicable)
- **Analysis Baseline**: (short code, if applicable)
- **Tool Outputs**: (list tool names and version info)

## Results

<!-- Summarize pass/fail outcome and key metrics. Use a table for multiple checks. -->
| Check | Metric | Threshold | Actual | Result |
|-------|--------|-----------|--------|--------|
| (check name) | (metric) | (threshold) | (value) | Pass/Fail |

**Overall Verdict**: (PASS / FAIL / PASS_WITH_WARNINGS)

## Evidence

<!-- Link to raw logs, reports, or artifacts that substantiate the results above. -->
- (path or reference to evidence artifact)