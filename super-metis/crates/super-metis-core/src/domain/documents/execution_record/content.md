# {{ title }}

## Execution Summary

<!-- What was executed, by whom (human or agent), and what was the outcome. -->
- **Initiating Artifact**: (short code of the task/story that triggered execution)
- **Execution Mode**: (autonomous / supervised / manual)
- **Started**: (timestamp)
- **Completed**: (timestamp)
- **Outcome**: (success / partial / failed / escalated)

## Context & Sources

<!-- Every context source consulted during execution. This is the audit trail. -->
- **Architecture Reference**: (short code)
- **Rules Consulted**: (list of RulesConfig short codes)
- **Durable Notes Fetched**: (list of DurableInsightNote short codes)
- **Baselines Referenced**: (list of AnalysisBaseline short codes)

## Tools & Validations

<!-- Tools run during execution and their results. -->
| Tool | Command/Action | Files Touched | Result |
|------|---------------|---------------|--------|
| (tool name) | (what was run) | (count or list) | pass/fail |

## Decisions & Escalations

<!-- Decisions made during execution and any escalations to human review. -->

### Decisions Made
- (decision description -- rationale for the choice made)

### Escalations
- (what was escalated, why, and to whom)

### Overrides Applied
- (any governance overrides used, with justification)

## Notes

<!-- Anything else relevant about this execution run. -->
(Additional observations, performance notes, or suggestions for future runs.)