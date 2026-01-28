# k1s0 Go Framework

This directory contains the Go backend framework packages for the k1s0 platform.

## Overview

The k1s0 Go framework provides a set of reusable packages that implement Clean Architecture patterns and k1s0 conventions. These packages are designed to provide feature parity with the Rust framework crate.

## Packages

### Tier 1 (Foundation)

These packages have no framework dependencies.

| Package | Description |
|---------|-------------|
| **k1s0-error** | Clean Architecture error types with domain, application, and presentation layers |
| **k1s0-config** | YAML configuration loading with secret file reference support |
| **k1s0-validation** | Input validation with REST/gRPC error conversion |
| **k1s0-observability** | Structured logging, tracing, and metrics |

### Tier 2 (Infrastructure)

These packages may depend on Tier 1 packages.

| Package | Description |
|---------|-------------|
| **k1s0-grpc-server** | gRPC server foundation with interceptors |
| **k1s0-health** | Kubernetes health check support (liveness/readiness/startup) |

## Tier Dependency Rules

```
Tier 1 (Foundation) - No framework dependencies
|-- k1s0-error
|-- k1s0-config
|-- k1s0-validation
|-- k1s0-observability

Tier 2 (Infrastructure) - Can depend on Tier 1 only
|-- k1s0-grpc-server -> k1s0-error, k1s0-observability
|-- k1s0-health      -> (standalone)
```

## Installation

Use Go modules to import the packages:

```go
import (
    k1s0error "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error"
    k1s0config "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-config"
    k1s0obs "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
    k1s0val "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-validation"
    k1s0health "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-health"
    k1s0grpc "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-grpc-server"
)
```

## Quick Start

### Error Handling

```go
// Domain layer: transport-independent
domainErr := k1s0error.NotFound("User", "user-123")

// Application layer: add error code
appErr := k1s0error.NewAppError(domainErr, k1s0error.NewErrorCode("user.not_found"))

// Presentation layer: convert to HTTP/gRPC
httpErr := appErr.ToHTTPError()  // Returns RFC 7807 ProblemDetails
grpcErr := appErr.ToGRPCError()  // Returns gRPC status with metadata
```

### Configuration

```go
// Create config options
opts := k1s0config.NewConfigOptions("dev").
    WithConfigPath("config/dev.yaml").
    WithSecretsDir("/var/run/secrets/k1s0")

// Load configuration
loader, err := k1s0config.NewConfigLoader(opts)
if err != nil {
    log.Fatal(err)
}

var config AppConfig
if err := loader.Load(&config); err != nil {
    log.Fatal(err)
}

// Resolve secrets from files
password, err := loader.ResolveSecretFile(config.DB.PasswordFile, "db.password_file")
```

### Observability

```go
// Create configuration
obsConfig, err := k1s0obs.NewConfigBuilder().
    ServiceName("user-service").
    Env("dev").
    Version("1.0.0").
    Build()

// Create logger
logger, err := k1s0obs.NewLogger(obsConfig)

// Log with context
reqCtx := obsConfig.NewRequestContext()
ctx := reqCtx.ToContext(context.Background())
logger.Info(ctx, "User created", zap.String("user_id", "123"))
// Output: {"timestamp":"...","level":"INFO","service.name":"user-service","trace.id":"...","message":"User created","user_id":"123"}
```

### Validation

```go
type CreateUserRequest struct {
    Name  string `json:"name" validate:"required,min=1,max=100"`
    Email string `json:"email" validate:"required,email"`
    Age   int    `json:"age" validate:"gte=0,lte=150"`
}

v := k1s0val.New()
req := &CreateUserRequest{Name: "", Email: "invalid"}

if errs := v.Validate(req); errs != nil && errs.HasErrors() {
    // For REST APIs
    problem := errs.ToProblemDetails()

    // For gRPC APIs
    details := errs.ToGRPCDetails()
}
```

### Health Checks

```go
checker := k1s0health.NewChecker().
    WithServiceName("user-service").
    AddComponent("database", func(ctx context.Context) *k1s0health.ComponentHealth {
        // Check database connection
        if err := db.Ping(ctx); err != nil {
            return k1s0health.Unhealthy("database", err.Error())
        }
        return k1s0health.Healthy("database")
    })

handler := k1s0health.NewProbeHandler(checker)
handler.RegisterHandlers(mux, "/healthz")
```

### gRPC Server

```go
config := k1s0grpc.NewConfig().
    WithPort(50051).
    WithServiceName("user-service").
    WithReflection(true)

server, err := k1s0grpc.NewServer(config, obsConfig)
if err != nil {
    log.Fatal(err)
}

// Register your services
pb.RegisterUserServiceServer(server.GRPCServer(), &userService{})

// Start the server
if err := server.Start(); err != nil {
    log.Fatal(err)
}
```

## k1s0 Conventions

### No Environment Variables

Environment variable usage is prohibited. Configuration must come from YAML files:

```go
// PROHIBITED
os.Getenv("DATABASE_HOST")

// CORRECT
config.Database.Host  // from config/dev.yaml
```

### Secret References

Secrets must be referenced via `*_file` suffix keys:

```yaml
# config/dev.yaml
database:
  host: localhost
  password_file: db_password  # Reference to /var/run/secrets/k1s0/db_password
```

### Error Code Format

```
{service_name}.{category}.{reason}
```

Examples: `auth.invalid_credentials`, `user.not_found`, `db.conflict`

### Required Log Fields

All log entries automatically include:
- `timestamp` - ISO 8601 format
- `level` - DEBUG/INFO/WARN/ERROR
- `message` - Log message
- `service.name` - Service name
- `service.env` - Environment (dev/stg/prod)
- `trace.id` - Distributed trace ID
- `request.id` - Request ID

## Development

### Building

```bash
cd framework/backend/go
go build ./...
```

### Testing

```bash
cd framework/backend/go
go test ./...
```

### Workspace

This directory uses Go workspaces for local development:

```bash
go work use ./k1s0-error ./k1s0-config ./k1s0-observability ./k1s0-validation ./k1s0-health ./k1s0-grpc-server
```

## External Dependencies

| Package | Dependency |
|---------|------------|
| k1s0-error | stdlib only |
| k1s0-config | gopkg.in/yaml.v3 |
| k1s0-validation | github.com/go-playground/validator/v10 |
| k1s0-observability | go.uber.org/zap, go.opentelemetry.io/otel |
| k1s0-grpc-server | google.golang.org/grpc |
| k1s0-health | stdlib only |

## License

See the root LICENSE file for license information.
