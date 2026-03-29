---
id: agent-console-output-parsing-and
level: initiative
title: "Agent Console Output Parsing and Reply Input Fix"
short_code: "SMET-I-0102"
created_at: 2026-03-29T01:10:00.860661+00:00
updated_at: 2026-03-29T01:10:00.860661+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
strategy_id: SMET-S-0002
initiative_id: agent-console-output-parsing-and
---

# Agent Console Output Parsing and Reply Input Fix Initiative

## Context

The dashboard's live agent console (session monitoring view) has two bugs that break the interactive session experience:

1. **Inconsistent output rendering**: Agent session events arrive in mixed formats — some as JSON objects and others as plain text strings. The dashboard is only rendering JSON-formatted events correctly; plain text responses are being lost/dropped, resulting in missing output in the console view.

2. **Reply input permanently disabled**: After the AI agent responds, the reply text field and action buttons remain disabled. The user cannot send follow-up messages or interact with the session, effectively making the console one-shot instead of interactive.

Both issues are in the control dashboard frontend, specifically in the session monitoring and intervention UI components.

## Goals & Non-Goals

**Goals:**
- Render all agent output formats correctly (JSON and plain text) in the console view
- Ensure the reply input field and buttons re-enable after receiving an agent response
- Maintain correct input state transitions throughout the session lifecycle

**Non-Goals:**
- Redesigning the console UI layout or styling
- Adding new output format types beyond what the API already sends
- Changes to the control API or machine runner event emission

## Detailed Design

TBD — requires investigation of:
- How session events are parsed and rendered in the dashboard (event type detection logic)
- What condition gates the reply input disabled state and why it fails to re-enable
- The SSE event stream format and any assumptions about event shape

## Alternatives Considered

TBD — pending discovery investigation.

## Implementation Plan

1. Investigate the event rendering pipeline to understand how JSON vs plain text events are handled
2. Investigate the reply input state management to find the disabled-state bug
3. Fix output rendering to handle both formats
4. Fix input re-enable logic
5. Manual testing across session lifecycle states