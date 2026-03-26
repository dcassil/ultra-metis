# JavaScript Server (Express/Fastify/Hono)

## Overview

Layered backend server architecture for Node.js using Express, Fastify, or Hono. Clear separation between HTTP handling and business logic. Suitable for REST APIs, GraphQL servers, and backend services.

## Structure

Routes define HTTP endpoints and delegate to handlers. Handlers orchestrate business logic by calling services. Services contain pure business rules and call repositories for data access. Repositories abstract database/external service interactions.

## Dependency Rules

- Routes depend on handlers only
- Handlers depend on services only
- Services depend on repositories and other services
- Repositories depend on database clients and external APIs
- No layer may depend on a layer above it

## Anti-Patterns

- Business logic in route handlers
- Direct database access from handlers
- Circular dependencies between services
- God services that handle multiple domains

## Quality Expectations

- ESLint clean with no-restricted-imports for layer enforcement
- TypeScript strict mode if using TS
- Unit tests for services, integration tests for routes

## Rules Seed Data

Generates rules for: layer-boundary enforcement, import restrictions, naming conventions, and test co-location requirements.