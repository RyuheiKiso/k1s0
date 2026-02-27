# k1s0-health ライブラリ設計

サーバーの `/healthz`・`/readyz` エンドポイント実装を共通化するライブラリ。

## 概要

PostgreSQL・Redis・Kafka・外部 HTTP エンドポイントへの依存チェックを統一 API で提供する。axum ルーターへの組み込みと、OpenTelemetry メトリクス連携をサポートする。各チェッカーは `HealthCheck` トレイトを実装し、`HealthChecker` に登録することで `/healthz`（死活確認）と `/readyz`（トラフィック受け入れ可否）の 2 エンドポイントを自動生成する。

sqlx・deadpool-redis・rdkafka はオプショナル feature として切り替え可能にし、不要な依存を持ち込まない設計とする。`HealthStatus` は `Healthy`・`Degraded`・`Unhealthy` の 3 段階で障害の重大度を表現する。

**配置先**: `regions/system/library/rust/health/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|------------|------|------|
| `HealthChecker` | 構造体 | 複数チェッカーを集約し healthz/readyz レスポンスを生成 |
| `HealthCheck` | トレイト | 個別ヘルスチェック実装インターフェース |
| `PostgresHealthCheck` | 構造体 | PostgreSQL 接続確認 |
| `RedisHealthCheck` | 構造体 | Redis PING 確認 |
| `KafkaHealthCheck` | 構造体 | Kafka ブローカー接続確認 |
| `HttpHealthCheck` | 構造体 | 外部 HTTP エンドポイントの GET 確認 |
| `HealthStatus` | enum | `Healthy`・`Degraded`・`Unhealthy` |
| `HealthResponse` | 構造体 | status + checks マップ（axum JSON レスポンス） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-health"
version = "0.1.0"
edition = "2021"

[features]
postgres = ["sqlx"]
redis = ["deadpool-redis"]
http = ["reqwest"]

[dependencies]
async-trait = "0.1"
axum = "0.7"
tokio = { version = "1", features = ["sync", "time"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }
deadpool-redis = { version = "0.18", optional = true }
reqwest = { version = "0.12", optional = true }
tracing = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
testcontainers = "0.23"
wiremock = "0.6"
```

**Cargo.toml への追加行**:

```toml
k1s0-health = { path = "../../system/library/rust/health", features = ["postgres", "redis"] }
```

**モジュール構成**:

```
health/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── checker.rs      # HealthChecker
│   ├── checks/
│   │   ├── mod.rs
│   │   ├── postgres.rs
│   │   ├── redis.rs
│   │   ├── kafka.rs
│   │   └── http.rs
│   └── response.rs     # HealthStatus・HealthResponse
└── Cargo.toml
```

**使用例**:

```rust
use axum::{routing::get, Router, Json};
use k1s0_health::{HealthChecker, PostgresHealthCheck, RedisHealthCheck};

// HealthChecker の構築
let health_checker = HealthChecker::new()
    .add_check("database", PostgresHealthCheck::new(pool.clone()))
    .add_check("redis", RedisHealthCheck::new(redis_client.clone()));

let health_checker = Arc::new(health_checker);

// axum ルーターへの組み込み
let app = Router::new()
    .route("/healthz", get(|| async { Json(json!({"status": "ok"})) }))
    .route("/readyz", get(move || {
        let checker = health_checker.clone();
        async move { checker.readyz().await }
    }));

// /readyz レスポンス例:
// {
//   "status": "Healthy",
//   "checks": {
//     "database": { "status": "Healthy", "duration_ms": 3 },
//     "redis":    { "status": "Healthy", "duration_ms": 1 }
//   }
// }
```

## Go 実装

**配置先**: `regions/system/library/go/health/`

```
health/
├── health.go
├── health_test.go
├── go.mod
└── go.sum
```

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
type Status string

const (
    StatusHealthy   Status = "healthy"
    StatusDegraded  Status = "degraded"
    StatusUnhealthy Status = "unhealthy"
)

type CheckResult struct {
    Status  Status
    Message string
}

type HealthResponse struct {
    Status    Status
    Checks    map[string]CheckResult
    Timestamp time.Time
}

type HealthCheck interface {
    Name() string
    Check(ctx context.Context) error
}

type Checker struct{}

func NewChecker() *Checker
func (c *Checker) Add(check HealthCheck)
func (c *Checker) RunAll(ctx context.Context) HealthResponse
func (c *Checker) Readyz(ctx context.Context) HealthResponse
func (c *Checker) Healthz() map[string]string
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/health/`

```
health/
├── package.json        # "@k1s0/health", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # HealthChecker, HealthCheck, HealthStatus, HealthResponse
└── __tests__/
    └── health.test.ts
```

**主要 API**:

```typescript
export type HealthStatus = 'Healthy' | 'Degraded' | 'Unhealthy';

export interface HealthCheckResult {
  status: HealthStatus;
  durationMs: number;
  error?: string;
}

export interface HealthResponse {
  status: HealthStatus;
  checks: Record<string, HealthCheckResult>;
}

export interface HealthCheck {
  check(): Promise<HealthStatus>;
}

export class HealthChecker {
  addCheck(name: string, check: HealthCheck): this;
  readyz(): Promise<HealthResponse>;
  healthz(): Promise<{ status: 'ok' }>;
}

export class HttpHealthCheck implements HealthCheck {
  constructor(url: string, timeoutMs?: number);
  check(): Promise<HealthStatus>;
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/health/`

```
health/
├── pubspec.yaml        # k1s0_health
├── analysis_options.yaml
├── lib/
│   ├── health.dart
│   └── src/
│       ├── checker.dart    # HealthChecker
│       ├── check.dart      # HealthCheck abstract
│       ├── response.dart   # HealthStatus・HealthResponse
│       └── checks/
│           └── http.dart   # HttpHealthCheck
└── test/
    └── health_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  http: ^1.3.0
```

**使用例**:

```dart
import 'package:k1s0_health/health.dart';

final checker = HealthChecker()
  ..addCheck(HttpHealthCheck(name: 'api-gateway', url: 'https://api.example.com/healthz'));

final response = await checker.readyz();
print(response.status); // HealthStatus.healthy
```

**カバレッジ目標**: 90%以上

## テスト戦略

**ユニットテスト** (`#[cfg(test)]`):
- `InMemory` スタブを `HealthCheck` トレイト実装として用意し、`Healthy`・`Degraded`・`Unhealthy` 各状態を返すモックで `HealthChecker` の集約ロジックを検証
- 全チェッカーが `Healthy` → レスポンス `status` が `Healthy` になることを確認
- 1 つでも `Unhealthy` → レスポンス `status` が `Unhealthy` になることを確認
- タイムアウト超過時に `Degraded` を返すことを確認

**統合テスト** (testcontainers):
- `PostgresHealthCheck`: testcontainers で PostgreSQL コンテナを起動し実接続確認
- `RedisHealthCheck`: testcontainers で Redis コンテナを起動し PING 確認
- `HttpHealthCheck`: wiremock でモックサーバーを起動し 200・503 各応答を確認

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthy;
    struct AlwaysUnhealthy;

    #[async_trait]
    impl HealthCheck for AlwaysHealthy {
        async fn check(&self) -> HealthStatus { HealthStatus::Healthy }
        fn name(&self) -> &str { "always-healthy" }
    }

    #[async_trait]
    impl HealthCheck for AlwaysUnhealthy {
        async fn check(&self) -> HealthStatus { HealthStatus::Unhealthy }
        fn name(&self) -> &str { "always-unhealthy" }
    }

    #[tokio::test]
    async fn test_all_healthy() {
        let checker = HealthChecker::new()
            .add_check("svc", AlwaysHealthy);
        let resp = checker.readyz().await;
        assert_eq!(resp.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_one_unhealthy_degrades_overall() {
        let checker = HealthChecker::new()
            .add_check("ok", AlwaysHealthy)
            .add_check("bad", AlwaysUnhealthy);
        let resp = checker.readyz().await;
        assert_eq!(resp.status, HealthStatus::Unhealthy);
    }
}
```

## 関連ドキュメント

- [system-library-概要](../overview/概要.md) — ライブラリ一覧・テスト方針
- [system-library-cache設計](../data/cache設計.md) — k1s0-cache ライブラリ
- [system-library-correlation設計](correlation設計.md) — k1s0-correlation ライブラリ
