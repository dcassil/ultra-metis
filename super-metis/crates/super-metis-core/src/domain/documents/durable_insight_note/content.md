# {{ title }}

## Insight

<!-- Describe the reusable knowledge concisely. This should be immediately actionable by a reader or agent. -->
- **Category**: (hotspot warning / recurring failure / misleading naming / validation hint / subsystem quirk / performance trap / dependency risk)
- **Confidence**: (high / medium / low -- based on how many times this has been validated)
- **Summary**: (one-sentence description of the insight)

### Detail
(Full explanation of the insight, including examples of when it applies and what to do about it.)

## Scope

<!-- Be precise about where this insight is relevant. -->
- **Repo/Package**: (which repo or package this applies to)
- **Paths**: (specific file paths or glob patterns, e.g., `src/domain/**/*.rs`)
- **Symbols**: (specific functions, types, or modules, if applicable)

## Signals

<!-- How an agent or human can detect when this insight is relevant. -->
- (trigger condition, e.g., "When modifying files in src/domain/quality/")
- (another signal)

## Notes

<!-- When this insight is most useful, known exceptions, or related notes. -->
- **Related Notes**: (short codes of related DurableInsightNotes)
- **Exceptions**: (situations where this insight does NOT apply)
- **Last Validated**: (date when this insight was last confirmed to be accurate)