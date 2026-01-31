# k1s0 Android Framework

Shared Android framework packages for the k1s0 platform, built with Jetpack Compose, Kotlin Coroutines, and Material 3.

## Packages

| Package | Description |
|---------|-------------|
| k1s0-navigation | Navigation Compose wrapper with config-driven routing |
| k1s0-config | YAML configuration loading from Android assets |
| k1s0-http | Ktor Client wrapper with interceptors |
| k1s0-ui | Material 3 design system and form generator |
| k1s0-auth | JWT authentication client |
| k1s0-observability | Structured logging and tracing |
| k1s0-state | ViewModel and StateFlow utilities |
| k1s0-realtime | WebSocket/SSE client with reconnection |

## Build

```bash
./gradlew build
```

## Test

```bash
./gradlew test
```

## Tech Stack

- Kotlin 2.0, Jetpack Compose, Material 3
- Koin for dependency injection
- Ktor Client for HTTP
- kotlinx.serialization for JSON/YAML
- kotlinx.coroutines + Flow for async
- JUnit 5 + MockK for testing
