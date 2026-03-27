# Rust Web Service (Actix/Axum)

## Overview

Layered async web service architecture using Actix-web or Axum. HTTP handlers are thin extractors that delegate to domain services. Domain logic is framework-agnostic and tested without HTTP infrastructure. Suitable for REST APIs, microservices, and backend services.

## Structure

Handlers define HTTP endpoints using framework extractors and delegate to domain services. Services contain pure business rules and depend on repository traits for data access. Repositories implement persistence behind trait abstractions. Models define domain types shared across layers. Error types convert between domain errors and HTTP responses.

## Dependency Rules

- Handlers depend on services and extractors only
- Services depend on repository traits (not concrete implementations)
- Repositories implement traits defined in the domain layer
- Models are shared value types with no framework dependencies
- No layer may depend on a layer above it

## Anti-Patterns

- Business logic in handler functions
- Framework types (HttpRequest, Json<T>) leaking into services
- Concrete database types in service signatures instead of trait objects
- Missing error type conversions (unwrap in handlers)
- God modules combining routing, logic, and persistence

## Quality Expectations

- Clippy clean with no warnings
- All public types documented
- Unit tests for services using mock repositories
- Integration tests for handlers using test server