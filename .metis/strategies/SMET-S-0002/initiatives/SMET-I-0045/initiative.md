---
id: ultra-metis-work-and-notes
level: initiative
title: "Ultra-Metis Work and Notes Integration"
short_code: "SMET-I-0045"
created_at: 2026-03-17T19:56:57.459764+00:00
updated_at: 2026-03-17T19:56:57.459764+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: ultra-metis-work-and-notes
---

# Ultra-Metis Work and Notes Integration Initiative

## Context

Remote AI sessions should not be disconnected execution islands — they should be first-class participants in the Ultra-Metis planning and memory system. This initiative connects the Control Service and Machine Runner back into Ultra-Metis: sessions can be launched from work items, execution context is enriched with relevant notes and architecture guidance, session results flow back into the linked work item, and the note system is updated based on session findings.

This is the bridge between SMET-S-0001 (core engine) and SMET-S-0002 (remote ops). It requires the core engine's note system (SMET-I-0030) and execution records (SMET-I-0031) to be functional.

**Pre-requisites**: SMET-I-0038, SMET-I-0039, SMET-I-0040, SMET-I-0043 (session history for result handoff); plus SMET-S-0001 note system (SMET-I-0030) and execution records (SMET-I-0031).

**Components touched**: Control Service (work item linkage API, note fetch/score API, result handoff), Machine Runner (note loading at session start, architecture guidance injection, finding capture), Control Dashboard (work item selector at session start, linked work item display, note proposal review).

## Goals & Non-Goals

**Goals:**
- Session start flow includes a work item selector: attach an Ultra-Metis task, story, or initiative
- Session context enriched at start with relevant notes from the core engine note system
- Session context includes applicable architecture guidance and repo rules
- Machine Runner loads notes and architecture context into the AI session at startup
- Conflicts with architecture or policy flagged during or after the session
- Session result (summary, artifacts, next steps) flows back into the linked work item record
- Note feedback recorded: was a note helpful, unused, or harmful during the session
- Session proposes new notes from confirmed findings; proposals reviewable after session
- Note proposals and note feedback reviewable in dashboard after session ends
- Which rules/guidance were applied during a session is captured and explainable

**Non-Goals:**
- Modifying notes directly from the dashboard (curation happens in the core engine workflow)
- Automatically merging session results into work items without user review
- Architecture analysis or rule enforcement during session execution (that's SMET-I-0044 for remote policy; core engine handles architecture)

## Detailed Design

### Work Item Linkage
- Session creation API accepts optional `work_item_id` (Ultra-Metis short code: task, story, or initiative)
- Control Service stores the link; session detail view shows linked work item with title and short code
- On session completion: Control Service calls Ultra-Metis MCP to append session outcome to the linked work item's progress section

### Note and Architecture Context Loading (Machine Runner)
- On session start, Machine Runner calls Ultra-Metis MCP `fetch_notes` for the session's repo area
- Fetched notes injected into AI session context as a CLAUDE.md append or initial context message
- Architecture guidance (reference architecture for the repo) fetched from Ultra-Metis and included
- Relevant rules fetched and included as constraints in the initial session context

### Note Feedback Recording
- Machine Runner tracks which notes were referenced during the session (via session trace analysis or explicit AI signals)
- At session end, emits `NoteFeedback` events: `{note_id, feedback: helpful|meh|harmful}`
- Control Service forwards feedback to Ultra-Metis MCP `score_note` for each note
- Feedback is advisory — it informs the note system's self-pruning (SMET-I-0030 logic)

### Note Proposals
- During session, AI may emit structured note proposals (e.g., "I discovered X about module Y")
- Machine Runner captures these as `NoteProposal` events
- Control Service stores proposals; dashboard shows them in a "Proposed Notes" section on session detail
- User can accept (creates note via Ultra-Metis MCP) or dismiss each proposal

### Dashboard — Integration Views
- Session start: work item search/select field (search by title or short code)
- Session detail: "Linked Work Item" card showing title, short code, link to work item
- Session detail: "Proposed Notes" tab with accept/dismiss per proposal
- Session detail: "Note Feedback" summary (how many notes were helpful/meh/harmful)

## Multi-Tenancy Notes

### Work Item Scoping
- Work item linkage (`session.work_item_id`) is implicitly user-scoped because sessions are user-scoped
- The work item selector in the dashboard searches Ultra-Metis documents — those are already repo/project scoped; no additional user filtering needed at the linkage layer

### Note Proposals and Feedback
- `note_proposals` table: `user_id` foreign key — proposals belong to the session owner
- `note_feedback` records: `user_id` foreign key — feedback is attributed to the user whose session generated it
- **MVP**: all owned by `user_id=1`; correctly attributed when real users are added

### Future Multi-User Consideration
- When multiple developers share a team, note proposals from any team member's session could be visible to the team for curation — this would be a team-scoped view added later
- Note feedback from multiple users on the same note can be aggregated to improve pruning signal quality — the `user_id` attribution on feedback records makes this possible without schema changes

## Alternatives Considered

- **Inline note editing in dashboard**: richer but duplicates the core engine's note management workflow; rejected — proposals are reviewed in dashboard, curation happens in the engine
- **Automatic result handoff without user review**: simpler but risky — session outcomes may be partial or incorrect; rejected in favor of user-reviewed handoff
- **No context loading at session start**: simpler Machine Runner, but loses the core value of the notes system; rejected — fetching relevant notes at start is a primary use case

## Implementation Plan

1. Add `work_item_id` field to session creation API and storage
2. Implement result handoff: on session complete, call Ultra-Metis MCP to update linked work item
3. Implement note fetching in Machine Runner at session start (call Ultra-Metis MCP)
4. Implement context injection: notes + architecture + rules prepended to session context
5. Implement note feedback event capture and emission from Machine Runner
6. Implement note feedback forwarding to Ultra-Metis MCP from Control Service
7. Implement note proposal capture from session events
8. Build work item selector in session start flow (dashboard)
9. Build "Proposed Notes" tab in session detail
10. Build "Note Feedback" summary view
11. Integration test: start session with work item → complete → verify work item updated → verify notes scored