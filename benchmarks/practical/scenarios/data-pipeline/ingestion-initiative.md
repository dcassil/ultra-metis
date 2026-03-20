# Ingestion Stage Initiative

## Context

The ingestion stage is the entry point of the pipeline. It must accept events from multiple source types, validate their schema, and produce well-typed `Event` structs for downstream processing.

## Goals

- Implement the `Event` struct and `EventType` enum as the core typed data model.
- Build an `IngestionService` that accepts raw input and produces validated events.
- Support at least two source adapters: HTTP webhook and file watcher.
- Define `IngestionError` for all failure modes.
- Validate all incoming data against the event schema before forwarding.

## Non-Goals

- Enrichment logic (separate stage)
- Routing logic (separate stage)
- Persistent storage of raw events

## Acceptance Criteria

- Event struct includes all fields from the spec (id, source, event_type, payload, timestamp, metadata).
- IngestionService validates input and rejects malformed events with descriptive errors.
- At least two source adapter implementations.
- Unit tests cover valid input, invalid input, and edge cases.
