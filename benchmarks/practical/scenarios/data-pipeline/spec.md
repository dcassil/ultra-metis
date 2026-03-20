# Event-Driven Data Pipeline — Specification

## Event Schema

Events are the core data unit. Every event has:
- `id`: UUID v4
- `source`: String identifying the origin (e.g., "webhook", "file-watcher")
- `event_type`: Enum of known event types
- `payload`: Typed payload variant matching the event_type
- `timestamp`: UTC datetime
- `metadata`: Key-value map for tracing and routing hints

## Pipeline Stages

### 1. Ingestion
- Accepts events from multiple sources (HTTP webhook, file watcher)
- Validates schema and rejects malformed input with structured errors
- Assigns event IDs and timestamps if not present
- Outputs validated `Event` structs to the enrichment stage

### 2. Enrichment
- Receives validated events
- Looks up contextual data (e.g., user profile, geo data) from a pluggable provider trait
- Attaches enrichment data to event metadata
- Passes enriched events to routing

### 3. Routing
- Evaluates routing rules against event type and metadata
- Supports static rules (config-driven) and dynamic rules (trait-based)
- Fans out events to one or more sink channels

### 4. Sink
- Receives routed events
- Supports multiple sink implementations (file sink, stdout sink)
- Handles write failures with retry and dead-letter semantics
- Confirms delivery or reports failure

## Error Handling

Each stage defines its own error enum:
- `IngestionError`: InvalidSchema, SourceUnavailable, DuplicateEvent
- `EnrichmentError`: ProviderTimeout, MissingContext, SerializationError
- `RoutingError`: NoMatchingRule, ChannelFull
- `SinkError`: WriteFailure, RetryExhausted, DeadLettered

## Backpressure

- Bounded channels between stages with configurable capacity
- When a channel is full, the upstream stage receives a backpressure signal
- Configurable behavior: drop, block, or buffer-to-disk

## Testing Requirements

- Unit tests for each stage in isolation using mock inputs
- Integration test for the full pipeline with test events
- Error path tests for each error variant
- Backpressure test demonstrating correct behavior under load
