# k1s0 C# Backend Framework

Shared C# (ASP.NET Core) framework packages for k1s0 microservices.

## Packages

### Tier 1 - Core Foundation (no framework dependencies)

| Package | Description |
|---------|-------------|
| K1s0.Error | Unified error handling (RFC 7807 Problem Details) |
| K1s0.Config | YAML-based configuration management |
| K1s0.Validation | Input validation |

### Tier 2 - Technical Infrastructure (depends on Tier 1 only)

| Package | Description |
|---------|-------------|
| K1s0.Observability | OpenTelemetry-based logging, tracing, and metrics |
| K1s0.Grpc.Server | gRPC server foundation with interceptors |
| K1s0.Grpc.Client | gRPC client utilities |
| K1s0.Health | Health check endpoints (liveness/readiness/startup) |
| K1s0.Db | Database connection (Entity Framework Core) |
| K1s0.DomainEvent | Domain event publish/subscribe and outbox pattern |
| K1s0.Resilience | Circuit breaker, retry, timeout, bulkhead |
| K1s0.Cache | Redis caching (StackExchange.Redis) |

### Tier 3 - Business Logic Support (depends on Tier 1 and 2)

| Package | Description |
|---------|-------------|
| K1s0.Auth | JWT/OIDC authentication and policy-based authorization |

## Requirements

- .NET 8.0 SDK
- C# 12

## Development

```bash
# Build
dotnet build

# Test
dotnet test

# Format check
dotnet format --verify-no-changes
```

## Tier Dependency Rules

- Tier 1: No framework dependencies allowed
- Tier 2: May depend on Tier 1 only
- Tier 3: May depend on Tier 1 and Tier 2
