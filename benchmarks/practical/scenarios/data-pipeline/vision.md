# Event-Driven Data Pipeline

## Vision

Build a modular, event-driven data pipeline that ingests events from multiple sources, enriches them with contextual data, routes them based on content, and delivers them to appropriate sinks. The pipeline must handle backpressure gracefully, provide clear error boundaries at each stage, and use typed event schemas throughout.

## Key Objectives

1. **Typed Event Flow**: All events flow through strongly-typed structs with explicit schema versioning.
2. **Stage Isolation**: Each pipeline stage (ingestion, enrichment, routing, sink) is an independent module with defined input/output contracts.
3. **Error Boundaries**: Each stage owns its error types. Failures in one stage do not cascade uncontrolled into others.
4. **Backpressure**: The pipeline supports configurable buffering and backpressure signaling between stages.
5. **Observability**: Every stage emits structured metrics and traces.

## Constraints

- Rust implementation using async/await.
- No external message broker dependency for the first version — use in-process channels.
- Must support at least two event sources and two sink targets.
- Must include comprehensive unit and integration tests.
