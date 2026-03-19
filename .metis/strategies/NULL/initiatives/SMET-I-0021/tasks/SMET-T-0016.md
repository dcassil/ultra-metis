---
id: implement-pluggable-tool-output
level: task
title: "Implement pluggable tool output parser trait and ESLint JSON parser"
short_code: "SMET-T-0016"
created_at: 2026-03-16T21:20:11.194622+00:00
updated_at: 2026-03-16T21:28:03.909439+00:00
parent: SMET-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: SMET-I-0021
---

# Implement pluggable tool output parser trait and ESLint JSON parser

## Objective

Define the pluggable ToolOutputParser trait and shared data types (MetricEntry, FindingEntry, ParsedToolOutput), then implement the first concrete parser for ESLint JSON output format.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] ToolOutputParser trait: `fn tool_name() -> &str`, `fn parse(input: &str) -> Result<ParsedToolOutput>`
- [ ] MetricEntry struct: metric_name, value (f64), unit, file_path (optional)
- [ ] FindingEntry struct: rule_id, severity (Error/Warning/Info), message, file_path, line, column
- [ ] ParsedToolOutput struct: tool_name, timestamp, metrics (Vec<MetricEntry>), findings (Vec<FindingEntry>), summary (HashMap<String, f64>)
- [ ] Severity enum: Error, Warning, Info with Display/FromStr
- [ ] EslintParser: parses ESLint JSON output format into ParsedToolOutput
- [ ] Unit tests with realistic ESLint JSON fixtures
- [ ] All existing tests still pass

## Implementation Notes

### New module: `src/domain/quality/`
- `mod.rs` — module declarations
- `types.rs` — MetricEntry, FindingEntry, ParsedToolOutput, Severity
- `parser.rs` — ToolOutputParser trait
- `parsers/mod.rs` — parser module
- `parsers/eslint.rs` — EslintParser

### ESLint JSON Format
ESLint `--format json` produces an array of file results, each with `filePath`, `messages[]` containing `ruleId`, `severity` (1=warn, 2=error), `message`, `line`, `column`, and summary stats.

## Progress

*Updated during implementation*