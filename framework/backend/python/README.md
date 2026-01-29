# k1s0 Python Backend Framework

Shared Python framework packages for k1s0 microservices.

## Packages

### Tier 1 (No framework dependencies)

| Package | Description |
|---------|-------------|
| k1s0-error | Unified error handling with RFC 7807 Problem Details |
| k1s0-config | YAML-based configuration management |
| k1s0-validation | Input validation built on Pydantic |

### Tier 2 (May depend on Tier 1)

| Package | Description |
|---------|-------------|
| k1s0-observability | OpenTelemetry-based logging, tracing, and metrics |
| k1s0-grpc-server | gRPC server foundation with interceptors |
| k1s0-grpc-client | gRPC client utilities and channel factory |
| k1s0-health | Health check endpoints (liveness and readiness) |
| k1s0-db | Async SQLAlchemy database utilities |

## Requirements

- Python 3.12+
- uv (package manager)

## Development

```bash
# Install all packages in development mode
uv sync

# Run tests
uv run pytest

# Type checking
uv run mypy packages/

# Linting
uv run ruff check .
```

## Tier Dependency Rules

- Tier 1: No framework dependencies allowed
- Tier 2: May depend on Tier 1 only
