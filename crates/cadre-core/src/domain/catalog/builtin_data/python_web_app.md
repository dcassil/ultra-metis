# Python Web Application (FastAPI/Django)

## Overview

Layered web application architecture using FastAPI or Django. Routes define HTTP endpoints and delegate to service functions. Services contain business logic and depend on repository/ORM abstractions for data access. Schemas define request/response models separately from database models. Suitable for REST APIs, web backends, and microservices.

## Structure

Routes (or views) define HTTP endpoints with input validation via Pydantic models or Django serializers. Services contain pure business logic operating on domain types. Repositories or ORM managers handle database access. Schemas define API contracts. Models define database/ORM representations. Config handles settings via environment variables.

## Dependency Rules

- Routes depend on services and schemas only
- Services depend on repositories and domain models
- Repositories depend on ORM models and database clients
- Schemas are independent data contracts with no ORM dependencies
- No layer may depend on a layer above it

## Anti-Patterns

- Business logic in route/view functions
- ORM queries directly in routes
- Mixing Pydantic schemas with ORM models
- Circular imports between modules
- Missing type hints on public functions

## Quality Expectations

- Ruff or flake8 clean with no warnings
- Type hints on all public functions (mypy/pyright compatible)
- Unit tests for services, integration tests for routes
- Test fixtures using pytest with dependency injection