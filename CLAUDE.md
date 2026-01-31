# CLAUDE.md - AI Assistant Guide for k1s0

This document provides comprehensive guidance for AI assistants working with the k1s0 codebase.

## Project Overview

**k1s0** is a unified development platform for microservices that enables teams to focus on implementing business logic while the platform handles boilerplate, code generation, and convention enforcement.

### Core Features

- **Service scaffold generation**: Templates generate consistent directory structures
- **Development convention enforcement**: 11 lint rules detect violations with some auto-fix
- **Safe template upgrades**: Managed/protected area separation prevents breaking changes

### Philosophy

- Monorepo containing Framework, Templates, and CLI
- Separates "managed" areas (generated/protected) from "unmanaged" areas (editable by teams)
- Enforces development conventions through automated linting
- Provides safe template upgrades without breaking team code

## Technology Stack

| Layer | Technologies |
|-------|-------------|
| **CLI** | Rust 1.85+ (clap 4.5, Tera 1.19, tokio) |
| **Backend** | Rust (axum, tokio) + Go + C# (ASP.NET Core 8.0) + Python (FastAPI) + Kotlin (Ktor 3.x, Exposed, Koin) |
| **Frontend** | React (Material-UI, Zod, TypeScript 5.5) + Flutter (Dart) + Android (Jetpack Compose, Material 3) |
| **Database** | PostgreSQL |
| **Cache** | Redis |
| **Observability** | OpenTelemetry (OTEL Collector) |
| **API Protocols** | gRPC (internal), REST/OpenAPI (external) |
| **Contract Management** | buf (proto linting/breaking changes), Spectral (OpenAPI linting) |
| **Package Managers** | Cargo (Rust), pnpm 9.15.4+ (Node), NuGet (.NET), uv (Python), Gradle (Kotlin/Android) |

## Architecture: Three-Layer Structure

k1s0 uses a three-layer architecture:

```
framework (technical foundation) -> domain (business domain) -> feature (individual functions)
```

| Layer | Location | Purpose |
|-------|----------|---------|
| **framework** | `framework/` | Technical infrastructure (logging, config, error handling, DB connection) |
| **domain** | `domain/` | Business domain logic shared across features (entities, value objects, domain services) |
| **feature** | `feature/` | Concrete use case implementations (REST/gRPC endpoints, UI) |

### Dependency Rules

- feature -> domain: Allowed (with version constraints)
- feature -> framework: Allowed
- domain -> framework: Allowed
- domain -> domain: Allowed (but circular dependencies are prohibited)
- framework -> domain: **Prohibited** (framework is the bottom layer)
- framework -> feature: **Prohibited**

## Directory Structure

```
k1s0/
├── CLI/                          # Rust CLI tool (0.1.0)
│   ├── crates/
│   │   ├── k1s0-cli/            # Main CLI executable (clap-based)
│   │   ├── k1s0-generator/      # Template engine & Lint engine
│   │   └── k1s0-lsp/            # LSP server (completions, hover)
│   ├── templates/               # 5 service templates
│   │   ├── backend-rust/        # Rust backend scaffold
│   │   ├── backend-go/          # Go backend scaffold
│   │   ├── backend-csharp/      # C# backend scaffold
│   │   ├── backend-python/     # Python backend scaffold
│   │   ├── backend-kotlin/      # Kotlin backend scaffold
│   │   ├── frontend-react/      # React app scaffold
│   │   ├── frontend-flutter/    # Flutter app scaffold
│   │   └── frontend-android/    # Android app scaffold (Kotlin)
│   └── schemas/                 # JSON Schema definitions
│
├── framework/                    # Shared libraries & services (Layer 1)
│   ├── backend/
│   │   ├── rust/
│   │   │   ├── crates/          # 11 shared Rust crates
│   │   │   └── services/        # Common microservices (auth, config, endpoint)
│   │   ├── go/
│   │   ├── csharp/              # C# NuGet packages
│   │   ├── python/              # Python packages (uv)
│   │   └── kotlin/              # Kotlin packages (Gradle)
│   └── frontend/
│       ├── react/packages/      # 8 React packages
│       ├── flutter/packages/    # Flutter packages
│       └── android/packages/    # Android packages
│
├── domain/                      # Business domain libraries (Layer 2)
│   ├── backend/
│   │   ├── rust/{domain_name}/  # Rust domain crates
│   │   ├── go/{domain_name}/    # Go domain modules
│   │   ├── csharp/{domain_name}/ # C# domain projects
│   │   ├── python/{domain_name}/ # Python domain packages
│   │   └── kotlin/{domain_name}/ # Kotlin domain modules
│   └── frontend/
│       ├── react/{domain_name}/ # React domain packages
│       ├── flutter/{domain_name}/ # Flutter domain packages
│       └── android/{domain_name}/ # Android domain modules
│
├── feature/                     # Individual feature services (Layer 3)
│   ├── backend/
│   │   ├── rust/{feature_name}/
│   │   ├── go/{feature_name}/
│   │   ├── csharp/{feature_name}/
│   │   ├── python/{feature_name}/
│   │   └── kotlin/{feature_name}/
│   ├── frontend/
│   │   ├── react/{feature_name}/
│   │   ├── flutter/{feature_name}/
│   │   └── android/{feature_name}/
│   └── database/
│
├── bff/                         # Backend-for-Frontend layer (optional)
│
├── docs/                        # Comprehensive documentation
│   ├── adr/                     # Architecture Decision Records
│   ├── architecture/            # System design docs
│   ├── conventions/             # Development rules
│   ├── design/                  # Technical design docs
│   └── operations/              # Deployment & runbooks
│
├── scripts/                     # Build & verification scripts
├── work/                        # Draft documents
└── .github/workflows/           # 12 CI/CD workflows
```

## Build & Development Commands

### Rust CLI

```bash
# Navigate to CLI directory
cd CLI

# Build
cargo build

# Run
cargo run -- --help

# Test all crates
cargo test --all

# Lint with Clippy (pedantic)
cargo clippy --all-targets --all-features -- -D warnings

# Check without building
cargo check
```

### Go Backend

```bash
# Navigate to Go framework directory
cd framework/backend/go

# Build all modules
for dir in */; do
  if [ -f "${dir}go.mod" ]; then
    cd "$dir" && go build ./... && cd ..
  fi
done

# Test all modules with race detector
for dir in */; do
  if [ -f "${dir}go.mod" ]; then
    cd "$dir" && go test -v -race ./... && cd ..
  fi
done

# Format check
gofmt -l .

# Static analysis
go vet ./...

# Lint (requires golangci-lint)
golangci-lint run --timeout=5m ./...

# Verify dependencies
go mod verify && go mod tidy
```

### Frontend (React)

```bash
# Install dependencies
pnpm install

# Build all packages
pnpm build

# Type check
pnpm typecheck

# Lint
pnpm lint
```

### Frontend (Flutter)

```bash
# Get dependencies
dart pub get

# Bootstrap with melos (if using monorepo tools)
melos bootstrap

# Analyze code
dart analyze

# Run build_runner (for code generation)
dart run build_runner build
```

### Kotlin Backend

```bash
# Navigate to Kotlin framework directory
cd framework/backend/kotlin

# Build
./gradlew build

# Test
./gradlew test

# Lint
./gradlew ktlintCheck

# Static analysis
./gradlew detekt
```

### Frontend (Android)

```bash
# Navigate to Android framework directory
cd framework/frontend/android

# Build debug APK
./gradlew assembleDebug

# Run unit tests
./gradlew testDebugUnitTest

# Lint (ktlint)
./gradlew ktlintCheck

# Android Lint
./gradlew lintDebug
```

### Docker

```bash
# Docker イメージをビルド
k1s0 docker build

# カスタムタグでビルド
k1s0 docker build --tag my-app:1.0

# docker compose でローカル環境を起動
k1s0 docker compose up -d --build

# docker compose サービスを停止（ボリューム削除）
k1s0 docker compose down -v

# コンテナ状態の確認
k1s0 docker status
```

### API Contract Validation

```bash
# Protocol Buffer linting
buf lint

# Check for breaking changes
buf breaking --against '.git#branch=main'

# Format check
buf format --exit-code
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `k1s0 init` | Initialize repository (`.k1s0/` directory) |
| `k1s0 new-feature --type <type> --name <name>` | Generate service scaffold (type: backend-rust, backend-go, backend-csharp, backend-python, backend-kotlin, frontend-react, frontend-flutter, frontend-android) |
| `k1s0 new-domain --type <type> --name <name>` | Generate domain scaffold (type: backend-rust, backend-go, backend-csharp, backend-python, backend-kotlin, frontend-react, frontend-flutter, frontend-android) |
| `k1s0 new-screen --type <type> --screen <id>` | Generate frontend screen |
| `k1s0 lint` | Check conventions |
| `k1s0 lint --fix` | Auto-fix violations |
| `k1s0 upgrade --check` | Show changes without applying |
| `k1s0 upgrade` | Apply template updates |
| `k1s0 doctor` | Check development environment health |
| `k1s0 doctor --json` | Output environment check as JSON |
| `k1s0 completions` | Generate shell completion scripts |
| `k1s0 domain-list` | List all domains |
| `k1s0 domain-version --name <name>` | Show/update domain version |
| `k1s0 domain-dependents --name <name>` | Show features depending on domain |
| `k1s0 domain-impact --name <name>` | Analyze version upgrade impact |
| `k1s0 feature-update-domain --name <name>` | Update feature's domain dependency |
| `k1s0 registry` | Template registry operations |
| `k1s0 domain-catalog` | Show domain catalog with dependency status |
| `k1s0 domain-graph` | Output domain dependency graph (Mermaid/DOT) |
| `k1s0 docker build` | Build Docker image (`--tag`, `--no-cache`, `--http-proxy`) |
| `k1s0 docker compose up` | Start docker compose services (`-d`, `--build`) |
| `k1s0 docker compose down` | Stop docker compose services (`-v`) |
| `k1s0 docker compose logs` | Show docker compose logs (`-f`, `<service>`) |
| `k1s0 docker status` | Show container status (`--json`) |
| `k1s0 playground start` | Start playground environment (`--type`, `--mode`, `--with-grpc`, `--with-db`, `--with-cache`, `--port-offset`) |
| `k1s0 playground stop` | Stop and remove playground (`--name`, `-v`, `-y`) |
| `k1s0 playground status` | Show running playgrounds (`--json`) |
| `k1s0 playground list` | List available playground templates |
| `k1s0 migrate analyze` | Analyze existing project for k1s0 compliance (`--path`, `--type`, `--json`, `--verbose`) |
| `k1s0 migrate plan` | Generate migration plan (`--path`, `--name`, `--type`, `--output`, `--dry-run`) |
| `k1s0 migrate apply` | Apply migration plan (`--path`, `--plan`, `--phase`, `--dry-run`, `--yes`, `--skip-backup`) |
| `k1s0 migrate status` | Show migration progress (`--path`, `--plan`, `--json`) |

### Interactive Mode

Commands `new-feature`, `new-domain`, `new-screen`, and `init` support interactive mode:

```bash
# Run without arguments to enter interactive mode (TTY required)
k1s0 new-feature

# Force interactive mode with -i flag
k1s0 new-feature -i

# Provide partial arguments, rest will be prompted interactively
k1s0 new-feature --type backend-rust
```

### Common Options

```
-v, --verbose      # Detailed output
-i, --interactive  # Force interactive mode (requires TTY)
-y, --yes          # Skip confirmation prompts (new-feature, new-domain, new-screen, upgrade)
--skip-doctor      # Skip environment health check (new-feature, init)
--no-color         # Disable ANSI colors
--json             # JSON format output
```

## Lint Rules (K001-K047)

### Manifest & Structure Rules (K001-K011)

| ID | Severity | Description | Auto-fix |
|----|----------|-------------|:--------:|
| K001 | Error | manifest.json does not exist | - |
| K002 | Error | manifest.json missing required keys | - |
| K003 | Error | manifest.json invalid values | - |
| K010 | Error | Required directory missing | Yes |
| K011 | Error | Required file missing | Yes |

### Code Quality Rules (K020-K029)

| ID | Severity | Description | Auto-fix |
|----|----------|-------------|:--------:|
| K020 | Error | Environment variable usage prohibited | - |
| K021 | Error | Secrets hardcoded in config YAML | - |
| K022 | Error | Clean Architecture dependency violation | - |
| K025 | Error | Config file naming convention violation (only default/dev/stg/prod allowed) | - |
| K026 | Error | Protocol type usage in Domain layer (HTTP/gRPC dependency) | - |
| K028 | Warning | Unused domain dependency declared in manifest.json | - |
| K029 | Error | Panic/unwrap/expect in production code (test files and entry points excluded) | - |

### Security Rules (K050-K053)

| ID | Severity | Description | Auto-fix |
|----|----------|-------------|:--------:|
| K050 | Error | SQL injection risk via string interpolation | - |
| K053 | Warning | Logging sensitive data (password, token, secret, etc.) | - |

### Infrastructure Rules (K060)

| ID | Severity | Description | Auto-fix |
|----|----------|-------------|:--------:|
| K060 | Warning | Dockerfile FROM with unpinned base image (:latest or no tag) | - |

### gRPC Retry Rules (K030-K032)

| ID | Severity | Description | Auto-fix |
|----|----------|-------------|:--------:|
| K030 | Warning | gRPC retry configuration detected | - |
| K031 | Warning | Retry config missing ADR reference | - |
| K032 | Warning | Retry config incomplete | - |

### Layer Dependency Rules (K040-K047)

| ID | Severity | Description | Auto-fix |
|----|----------|-------------|:--------:|
| K040 | Error | Layer dependency violation (e.g., framework depends on domain) | - |
| K041 | Error | Referenced domain not found | - |
| K042 | Error | Domain version constraint mismatch | - |
| K043 | Error | Circular dependency detected between domains | - |
| K044 | Warning | Using deprecated domain | - |
| K045 | Warning | min_framework_version not satisfied | - |
| K046 | Warning | Breaking changes impact detected | - |
| K047 | Error | Domain layer missing required version field | - |

## Critical Development Rules

### 1. No Environment Variables

**Prohibited patterns:**
- Rust: `std::env::var`, `std::env::var_os`, `std::env::vars`, `std::env::vars_os`, `std::env::set_var`, `std::env::remove_var`, `env::var(`, `env::var_os(`, `env::vars(`, `env::set_var(`, `env::remove_var(`, `dotenv`, `dotenvy`
- Go: `os.Getenv`, `os.LookupEnv`, `os.Setenv`, `os.Unsetenv`, `os.Environ`, `godotenv`
- TypeScript: `process.env`, `import.meta.env`, `dotenv`
- C#: `Environment.GetEnvironmentVariable`, `Environment.GetEnvironmentVariables`, `Environment.ExpandEnvironmentVariables`, `.AddEnvironmentVariables(`
- Python: `os.environ`, `os.getenv`, `os.putenv`, `os.unsetenv`, `load_dotenv`, `from dotenv`, `import dotenv`
- Kotlin: `System.getenv`, `System.getProperty`, `ProcessBuilder`, `dotenv`, `BuildConfig.`
- Dart: `Platform.environment`, `fromEnvironment`, `flutter_dotenv`

**Correct approach:** Use `config/*.yaml` files with k1s0-config library.

### 2. No Hardcoded Secrets

**Prohibited:** Direct values for password, secret, api_key, token, credential, private_key

**Correct approach:** Use `*_file` suffix referencing external files:
```yaml
database:
  password_file: /var/run/secrets/k1s0/db_password  # Reference, not value
```

### 3. Clean Architecture Dependency Rules

```
presentation -> application -> domain <- infrastructure
```

**Prohibited dependencies:**
- `domain -> application`
- `domain -> presentation`
- `domain -> infrastructure`
- `application -> presentation`

### 4. Naming Conventions

| Target | Convention | Example |
|--------|------------|---------|
| `{feature_name}` | kebab-case | `user-management`, `order-processing` |
| `{service_name}` | Same as feature_name | `user-management` |
| Framework common services | kebab-case + `-service` | `auth-service`, `config-service` |
| Environment | Fixed 4 values | `default`, `dev`, `stg`, `prod` |

### 5. Error Code Format

```
{service_name}.{category}.{reason}
```

Examples: `auth.invalid_credentials`, `user.not_found`, `db.conflict`

## Service Structure (Clean Architecture)

### Backend (Rust)

```
src/
├── domain/              # Business rules, entities, value objects
│   ├── entities/
│   ├── value_objects/
│   ├── repositories/    # Repository traits (ports)
│   └── services/        # Domain services
├── application/         # Use cases, application services
│   ├── usecases/
│   ├── services/
│   └── dtos/
├── infrastructure/      # Repository implementations, external I/O
│   ├── repositories/
│   ├── external/
│   └── persistence/
└── presentation/        # HTTP/gRPC handlers
    ├── grpc/
    ├── rest/
    └── middleware/
```

### Backend (Python)

```
src/{feature_name_snake}/
├── domain/              # Business rules, entities, value objects
│   ├── entities/
│   ├── value_objects/
│   ├── repositories/    # Repository abstract classes (ports)
│   └── services/        # Domain services
├── application/         # Use cases, application services
│   ├── usecases/
│   ├── services/
│   └── dtos/
├── infrastructure/      # Repository implementations, external I/O
│   ├── repositories/
│   ├── external/
│   └── persistence/
└── presentation/        # FastAPI routers, gRPC services
    ├── grpc/
    ├── rest/
    └── middleware/
```

### Backend (C#)

```
src/
├── {Name}.Domain/              # Business rules, entities, value objects
│   ├── Entities/
│   ├── ValueObjects/
│   ├── Repositories/           # Repository interfaces (ports)
│   └── Services/               # Domain services
├── {Name}.Application/         # Use cases, application services
│   ├── UseCases/
│   ├── Services/
│   └── Dtos/
├── {Name}.Infrastructure/      # Repository implementations, external I/O
│   ├── Repositories/
│   ├── External/
│   └── Persistence/
└── {Name}.Presentation/        # HTTP/gRPC handlers (ASP.NET Core)
    ├── Grpc/
    ├── Controllers/
    └── Middleware/
```

### Backend (Kotlin)

```
src/main/kotlin/{package}/
├── domain/              # Business rules, entities, value objects
│   ├── entities/
│   ├── valueobjects/
│   ├── repositories/    # Repository interfaces (ports)
│   └── services/        # Domain services
├── application/         # Use cases, application services
│   ├── usecases/
│   ├── services/
│   └── dtos/
├── infrastructure/      # Repository implementations, external I/O
│   ├── repositories/
│   ├── external/
│   └── persistence/
└── presentation/        # Ktor routes, gRPC services
    ├── grpc/
    ├── rest/
    └── middleware/
```

### Frontend (React)

```
src/
├── domain/              # Entities, value objects
├── application/         # Use cases, state management
├── infrastructure/      # API clients, repository implementations
└── presentation/        # Pages, components, hooks
```

### Frontend (Flutter)

```
lib/src/
├── domain/
├── application/
├── infrastructure/
└── presentation/        # Pages, widgets, controllers
```

### Frontend (Android)

```
app/src/main/kotlin/{package}/
├── domain/              # Business rules, entities, value objects
│   ├── entities/
│   ├── valueobjects/
│   ├── repositories/    # Repository interfaces (ports)
│   └── services/
├── application/         # Use cases, ViewModels
│   ├── usecases/
│   ├── services/
│   └── dtos/
├── infrastructure/      # Repository implementations, API clients
│   ├── repositories/
│   ├── external/
│   └── persistence/
└── presentation/        # Composable screens, navigation
    ├── screens/
    ├── components/
    └── theme/
```

## Required Files by Template

### backend-rust
- `Cargo.toml`, `src/main.rs`, `config/default.yaml`, `.k1s0/manifest.json`, `Dockerfile`, `.dockerignore`, `docker-compose.yml`
- Directories: `src/`, `src/domain/`, `src/application/`, `src/presentation/`, `src/infrastructure/`, `config/`, `deploy/`

### backend-go
- `go.mod`, `config/default.yaml`, `.k1s0/manifest.json`, `Dockerfile`, `.dockerignore`, `docker-compose.yml`
- Directories: `cmd/`, `internal/domain/`, `internal/application/`, `internal/presentation/`, `internal/infrastructure/`, `config/`, `deploy/`

### backend-csharp
- `{Name}.sln`, `src/{Name}.Presentation/{Name}.Presentation.csproj`, `config/default.yaml`, `.k1s0/manifest.json`
- Directories: `src/`, `src/{Name}.Domain/`, `src/{Name}.Application/`, `src/{Name}.Infrastructure/`, `src/{Name}.Presentation/`, `config/`, `deploy/`

### backend-python
- `pyproject.toml`, `config/default.yaml`, `.k1s0/manifest.json`
- Directories: `src/`, `src/{feature_name_snake}/domain/`, `src/{feature_name_snake}/application/`, `src/{feature_name_snake}/infrastructure/`, `src/{feature_name_snake}/presentation/`, `config/`, `deploy/`

### backend-kotlin
- `build.gradle.kts`, `settings.gradle.kts`, `src/main/kotlin/`, `config/default.yaml`, `.k1s0/manifest.json`, `Dockerfile`, `.dockerignore`, `docker-compose.yml`
- Directories: `src/`, `src/main/kotlin/*/domain/`, `src/main/kotlin/*/application/`, `src/main/kotlin/*/presentation/`, `src/main/kotlin/*/infrastructure/`, `config/`, `deploy/`

### frontend-react
- `package.json`, `tsconfig.json`, `.k1s0/manifest.json`, `Dockerfile`, `.dockerignore`, `docker-compose.yml`, `deploy/nginx.conf`
- Directories: `src/`, `src/domain/`, `src/application/`, `src/presentation/`, `public/`

### frontend-flutter
- `pubspec.yaml`, `.k1s0/manifest.json`
- Directories: `lib/`, `lib/src/domain/`, `lib/src/application/`, `lib/src/presentation/`

### frontend-android
- `build.gradle.kts`, `app/build.gradle.kts`, `app/src/main/AndroidManifest.xml`, `config/default.yaml`, `.k1s0/manifest.json`
- Directories: `app/src/main/kotlin/*/domain/`, `app/src/main/kotlin/*/application/`, `app/src/main/kotlin/*/presentation/`, `app/src/main/kotlin/*/infrastructure/`, `config/`

## Framework Crates (Rust Backend)

| Crate | Description | Tier |
|-------|-------------|------|
| k1s0-error | Unified error handling | 1 |
| k1s0-config | Config file management | 1 |
| k1s0-validation | Input validation (Zod-like) | 1 |
| k1s0-observability | Logging/tracing/metrics | 2 |
| k1s0-grpc-server | gRPC server foundation | 2 |
| k1s0-grpc-client | gRPC client utilities | 2 |
| k1s0-resilience | Retry/circuit breaker patterns | 2 |
| k1s0-health | Health check probes | 2 |
| k1s0-db | Database connection/transaction | 2 |
| k1s0-cache | Redis caching | 2 |
| k1s0-domain-event | Domain event publish/subscribe/outbox | 2 |
| k1s0-auth | Authentication/authorization | 3 |

**Tier dependency rules:**
- Tier 1: No framework dependencies
- Tier 2: Can depend on Tier 1 only
- Tier 3: Can depend on Tier 1 and 2

## Framework Packages (React Frontend)

| Package | Description |
|---------|-------------|
| @k1s0/navigation | Config-driven routing |
| @k1s0/config | YAML config management |
| @k1s0/api-client | HTTP/gRPC API client |
| @k1s0/ui | Design system (Material-UI based), DataTable (MUI DataGrid), Form Generator (Zod + react-hook-form) |
| @k1s0/shell | AppShell (Header/Sidebar/Footer) |
| @k1s0/auth-client | Client-side auth |
| @k1s0/observability | Frontend logging/analytics |
| @k1s0/realtime | WebSocket/SSE client with reconnection, heartbeat, offline queue |
| eslint-config-k1s0 | ESLint rules |
| tsconfig-k1s0 | Shared TypeScript config |

## Framework Packages (Python Backend)

| Package | Description | Tier |
|---------|-------------|------|
| k1s0-error | Unified error handling | 1 |
| k1s0-config | Config file management (YAML) | 1 |
| k1s0-validation | Input validation (Pydantic-based) | 1 |
| k1s0-observability | Logging/tracing/metrics (OpenTelemetry) | 2 |
| k1s0-grpc-server | gRPC server foundation (grpcio) | 2 |
| k1s0-grpc-client | gRPC client utilities | 2 |
| k1s0-health | Health check probes (FastAPI) | 2 |
| k1s0-db | Database connection/transaction (SQLAlchemy + asyncpg) | 2 |
| k1s0-domain-event | Domain event publish/subscribe and outbox pattern | 2 |
| k1s0-resilience | Circuit breaker, retry, timeout, bulkhead patterns | 2 |
| k1s0-cache | Redis caching and cache patterns | 2 |
| k1s0-auth | JWT/OIDC authentication and policy-based authorization | 3 |

**Tier dependency rules:** Same as Rust/Go/C# -- Tier 1 has no framework dependencies, Tier 2 can depend on Tier 1 only, Tier 3 can depend on Tier 1 and 2.

## Framework Packages (Flutter Frontend)

| Package | Description |
|---------|-------------|
| k1s0_navigation | Config-driven routing (go_router based) |
| k1s0_config | YAML config management |
| k1s0_http | HTTP client (Dio based) |
| k1s0_ui | Design system (Material 3), DataTable, Form Generator (schema-driven) |
| k1s0_auth | Authentication client (JWT/OIDC) |
| k1s0_observability | Structured logging, tracing |
| k1s0_state | Riverpod state management utilities |
| k1s0_realtime | WebSocket/SSE client with reconnection, heartbeat, offline queue |

## Framework Packages (C# Backend)

| Package | Description | Tier |
|---------|-------------|------|
| K1s0.Error | Unified error handling | 1 |
| K1s0.Config | Config file management | 1 |
| K1s0.Validation | Input validation | 1 |
| K1s0.Observability | Logging/tracing/metrics (OpenTelemetry) | 2 |
| K1s0.Grpc.Server | gRPC server foundation | 2 |
| K1s0.Grpc.Client | gRPC client utilities | 2 |
| K1s0.Health | Health check probes | 2 |
| K1s0.Db | Database connection/transaction (EF Core) | 2 |
| K1s0.DomainEvent | Domain event publish/subscribe and outbox pattern | 2 |
| K1s0.Resilience | Circuit breaker, retry, timeout, bulkhead patterns | 2 |
| K1s0.Cache | Redis caching and cache patterns (StackExchange.Redis) | 2 |
| K1s0.Auth | JWT/OIDC authentication and policy-based authorization | 3 |

**Tier dependency rules:** Same as Rust/Go -- Tier 1 has no framework dependencies, Tier 2 can depend on Tier 1 only, Tier 3 can depend on Tier 1 and 2.

## Framework Packages (Kotlin Backend)

| Package | Description | Tier |
|---------|-------------|------|
| k1s0-error | Unified error handling | 1 |
| k1s0-config | Config file management (YAML) | 1 |
| k1s0-validation | Input validation | 1 |
| k1s0-observability | Logging/tracing/metrics (OpenTelemetry) | 2 |
| k1s0-grpc-server | gRPC server foundation (grpc-kotlin) | 2 |
| k1s0-grpc-client | gRPC client utilities | 2 |
| k1s0-health | Health check probes (Ktor) | 2 |
| k1s0-db | Database (Exposed + HikariCP) | 2 |
| k1s0-domain-event | Domain event publish/subscribe/outbox | 2 |
| k1s0-resilience | Circuit breaker, retry, timeout | 2 |
| k1s0-cache | Redis caching (Lettuce) | 2 |
| k1s0-auth | JWT/OIDC auth (nimbus-jose-jwt) | 3 |

**Tier dependency rules:** Same as Rust/Go/C#/Python -- Tier 1 has no framework dependencies, Tier 2 can depend on Tier 1 only, Tier 3 can depend on Tier 1 and 2.

## Framework Packages (Android Frontend)

| Package | Description |
|---------|-------------|
| k1s0-navigation | Navigation Compose routing |
| k1s0-config | YAML config management |
| k1s0-http | Ktor Client HTTP |
| k1s0-ui | Material 3 design system |
| k1s0-auth | JWT auth client |
| k1s0-observability | Logging, tracing |
| k1s0-state | ViewModel + StateFlow utilities |
| k1s0-realtime | WebSocket/SSE client |

## CI/CD Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| cli.yml | Push to main/develop, CLI changes | Lint -> Test -> Integration Test -> Multi-platform Build |
| rust.yml | Push to main, framework/rust changes | Format check -> Clippy -> Tests -> Build |
| go.yml | Push to main/develop, Go changes | Format -> Lint -> Test -> Vet -> Mod verify -> Build |
| csharp.yml | Push to main, C# changes | Format -> Build -> Test |
| python.yml | Push to main, Python changes | Lint (Ruff) -> Format check -> Type check (mypy) -> Test (pytest) |
| frontend-react.yml | Push to main, React changes | Lint -> TypeCheck -> Test -> Build |
| kotlin.yml | Push to main/develop, Kotlin changes | ktlint -> detekt -> Build -> Test |
| frontend-android.yml | Push to main/develop, Android changes | ktlint -> detekt -> Android Lint -> Build -> Test |
| frontend-flutter.yml | Push to main, Flutter changes | Analyze -> Build |
| buf.yml | Push to main, proto changes | Lint -> Breaking changes check -> Format check |
| openapi.yml | Push to main, OpenAPI changes | Spectral linting |
| generation.yml | Push to main, contract changes | Fingerprint verification |
| docker.yml | Push to main/develop, Docker file changes | Docker build test (5 templates) |
| release-cli.yml | Semantic version tag | Validate -> Multi-platform build -> GH Release |
| release-crates.yml | Semantic version tag | Publish Rust crates to crates.io |
| release-npm.yml | Semantic version tag | Publish Node packages to npm |

## API Contract Management

### gRPC (Protocol Buffers)

- Location: `{service}/proto/*.proto`
- Generated code: `{service}/gen/` (not in Git)
- Tool: buf for linting and breaking change detection

**Allowed changes (backward compatible):**
- Adding optional fields
- Adding new services/methods
- Adding `deprecated = true`

**Prohibited changes (breaking):**
- Removing/renumbering fields
- Changing field types
- Removing oneof cases
- Changing service/method/package names

### REST (OpenAPI)

- Location: `{service}/openapi/openapi.yaml`
- Generated code: `{service}/openapi/gen/` (not in Git)
- Tool: Spectral for linting

## Error Handling Conventions

### Layer Responsibilities

1. **Domain layer**: Business failures (no HTTP/gRPC concepts)
2. **Application layer**: Classify errors, assign error_code
3. **Presentation layer**: Convert to REST/gRPC representation

### REST Error Response (RFC 7807)

```json
{
  "status": 404,
  "title": "Not Found",
  "detail": "User with ID 12345 was not found",
  "error_code": "user.not_found",
  "trace_id": "abc123def456"
}
```

### gRPC Status Codes

Use canonical codes only: `INVALID_ARGUMENT`, `UNAUTHENTICATED`, `PERMISSION_DENIED`, `NOT_FOUND`, `ALREADY_EXISTS`, `UNAVAILABLE`, `DEADLINE_EXCEEDED`, `INTERNAL`

## Configuration Conventions

### Priority (highest to lowest)

1. CLI arguments (`--config`, `--env`, `--secrets-dir`)
2. YAML files (`config/{env}.yaml`)
3. Database (`fw_m_setting` table)

### YAML File Structure

```
{service}/config/
├── default.yaml  # Common defaults
├── dev.yaml      # Development
├── stg.yaml      # Staging
└── prod.yaml     # Production
```

**Explicit environment selection required:** Always use `--env` flag, implicit selection prohibited.

## Important Documentation

| Document | Path | Description |
|----------|------|-------------|
| Main README | `README.md` | Project overview |
| CLI Design | `docs/design/cli/` | CLI architecture |
| Lint Design | `docs/design/lint/` | Lint rules detail (K001-K047) |
| Template Design | `docs/design/template/` | Template system |
| Framework Design | `docs/design/framework.md` | Library design |
| Domain Design | `docs/design/domain.md` | Domain layer design |
| Service Structure | `docs/conventions/service-structure.md` | Directory layout |
| Error Handling | `docs/conventions/error-handling.md` | Error conventions |
| API Contracts | `docs/conventions/api-contracts.md` | API management |
| Config & Secrets | `docs/conventions/config-and-secrets.md` | Configuration rules |
| Domain Boundaries | `docs/conventions/domain-boundaries.md` | Domain layer boundaries |
| Deprecation Policy | `docs/conventions/deprecation-policy.md` | Deprecation guidelines |
| Clean Architecture | `docs/architecture/clean-architecture.md` | CA principles |
| ADRs | `docs/adr/` | Architecture decisions |
| Domain Development | `docs/guides/domain-development.md` | Domain development guide |
| Domain Versioning | `docs/guides/domain-versioning.md` | Domain version management |
| Migration Guide | `docs/guides/migration-to-three-tier.md` | 2-tier to 3-tier migration |

## Available Specialized Agents

The `.claude/agents/` directory contains configurations for specialized Claude agents:

| Agent | Purpose |
|-------|---------|
| k1s0-orchestrator | Primary entry point for k1s0 tasks requiring coordination |
| k1s0-rust-dev | Rust development in CLI, framework, and services |
| k1s0-investigator | Codebase investigation and root cause analysis |
| k1s0-docs-writer | Documentation creation and updates |
| k1s0-lint-quality | Lint rules and code quality |
| k1s0-template-manager | Template management in CLI/templates/ |
| go-dev-k1s0 | Go backend development |
| frontend-dev | React and Flutter frontend development |
| api-designer | Protocol Buffers and OpenAPI design |
| cicd-manager | CI/CD pipeline management |

## Common Patterns

### Creating a New Feature Service

```bash
k1s0 new-feature --type backend-rust --name order-processing --with-grpc --with-db
```

This generates:
- Clean Architecture directory structure
- Manifest file (`.k1s0/manifest.json`)
- Required config files
- Kubernetes deployment templates

### Adding a New Screen (Frontend)

```bash
k1s0 new-screen --type frontend-react --screen user-profile
```

### Running Lint Check

```bash
# Check all rules
k1s0 lint

# Specific rules only
k1s0 lint --rules K020,K021,K022

# Auto-fix fixable issues
k1s0 lint --fix

# Strict mode (warnings as errors)
k1s0 lint --strict

# JSON output for CI
k1s0 lint --json
```

## Things to Avoid

1. **Never use environment variables** for configuration in application code
2. **Never hardcode secrets** in YAML or code
3. **Never violate Clean Architecture** dependency direction
4. **Never create common code** under `feature/` (move to `framework/`)
5. **Never import infrastructure from domain** layer
6. **Never add breaking changes** to APIs without ADR and migration plan
7. **Never skip lint checks** in CI
8. **Never store generated code** in Git

## Version Information

- k1s0 CLI: 0.1.0
- Rust Edition: 2024 (1.85+ required)
- Node: 20.x
- pnpm: 9.15.4+
- TypeScript: 5.5.4
