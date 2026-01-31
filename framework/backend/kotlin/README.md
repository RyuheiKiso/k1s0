# k1s0 Kotlin Framework

Kotlin backend framework packages for k1s0 microservices.

## Packages

### Tier 1 - Core Foundation (no framework dependencies)

| Package | Description |
|---------|-------------|
| k1s0-error | Unified error handling (RFC 7807) |
| k1s0-config | YAML configuration loading |
| k1s0-validation | Input validation DSL |

### Tier 2 - Technical Infrastructure (depends on Tier 1 only)

| Package | Description |
|---------|-------------|
| k1s0-observability | Logging, tracing, metrics (OpenTelemetry) |
| k1s0-grpc-server | gRPC server foundation |
| k1s0-grpc-client | gRPC client utilities |
| k1s0-health | Kubernetes health probes |
| k1s0-db | Database connection (Exposed + HikariCP) |
| k1s0-domain-event | Domain event publish/subscribe and outbox |
| k1s0-resilience | Circuit breaker, retry, timeout, bulkhead |
| k1s0-cache | Redis caching (Lettuce) |

### Tier 3 - Business Logic Support (depends on Tier 1 and 2)

| Package | Description |
|---------|-------------|
| k1s0-auth | JWT/OIDC authentication and authorization |

## Build

```bash
./gradlew build
```

## Test

```bash
./gradlew test
```

## Technology Stack

- Kotlin 2.0.21, JVM 21
- Ktor 3.0.3 (HTTP server)
- Exposed 0.57.0 (SQL)
- gRPC-Kotlin 1.4.1
- OpenTelemetry 1.45.0
- kotlinx.serialization, kotlinx.coroutines
