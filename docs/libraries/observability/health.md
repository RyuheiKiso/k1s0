# k1s0-health ライブラリ設計

サーバーの `/healthz`・`/readyz` エンドポイント実装を共通化するライブラリ。

## 概要

PostgreSQL・Redis・Kafka・外部 HTTP エンドポイントへの依存チェックを統一 API で提供する。各チェッカーは `HealthCheck` インターフェースを実装し、チェッカーに登録することで `/healthz`（死活確認）と `/readyz`（トラフィック受け入れ可否）の 2 エンドポイント用レスポンスを生成する。ライブラリはトランスポート非依存であり、HTTP フレームワーク（axum, Express 等）への組み込みは利用側サーバーの責務となる。

sqlx・deadpool-redis・rdkafka はオプショナル feature として切り替え可能にし、不要な依存を持ち込まない設計とする。`HealthStatus` は `Healthy`・`Degraded`・`Unhealthy` の 3 段階を定義する。ただし、現在の実装ではいずれの言語でも `Degraded` を自動生成するロジックは存在しない（カスタムチェッカーで利用者が明示的に返すことは可能）。

**配置先**:

| 言語 | パス |
|------|------|
| Go | `regions/system/library/go/health/` |
| Rust | `regions/system/library/rust/health/` |
| TypeScript | `regions/system/library/typescript/health/` |
| Dart | `regions/system/library/dart/health/` |

## ヘルスチェック実装の言語別対応状況

| チェックタイプ | Go | Rust | TypeScript | Dart |
|--------------|-----|------|-----------|------|
| HTTP | Yes | Yes (feature: `http`) | Yes | Yes |
| PostgreSQL | Yes | Yes (feature: `postgres`) | -- | -- |
| Redis | Yes | Yes (feature: `redis`) | -- | -- |
| Kafka | -- | Yes (feature: `kafka`) | -- | -- |

## 公開 API

### 共通型（全言語）

| 型・インターフェース | 種別 | 説明 |
|-------------------|------|------|
| `HealthStatus` | enum/type | `Healthy`・`Degraded`・`Unhealthy` |
| `CheckResult` | 構造体 | `status` + `message`（成功時 null/None/"OK"、失敗時エラー文字列） |
| `HealthResponse` | 構造体 | `status` + `checks` マップ + `timestamp` |
| `HealthCheck` | インターフェース/トレイト | `name` + `check()` メソッド（成功: void/Ok、失敗: throw/Err） |
| `HttpHealthCheck` | 構造体/クラス | 外部 HTTP エンドポイントの GET 確認 |

### Rust 固有

| 型・トレイト | 種別 | 説明 |
|------------|------|------|
| `HealthChecker` | トレイト | `add_check(&mut self, check: Box<dyn HealthCheck>)` |
| `CompositeHealthChecker` | 構造体 | `HealthChecker` トレイトの具象実装。複数チェッカーを集約し healthz/readyz レスポンスを生成 |
| `HealthError` | enum | `CheckFailed(String)`・`Timeout(String)` |
| `HealthzResponse` | 構造体 | `status: String`（常に `"ok"`） |
| `PostgresHealthCheck` | 構造体 (feature: `postgres`) | PostgreSQL 接続確認 |
| `RedisHealthCheck` | 構造体 (feature: `redis`) | Redis PING 確認 |
| `KafkaHealthCheck` | 構造体 (feature: `kafka`) | Kafka ブローカー接続確認 |

### Go 固有

| 型 | 種別 | 説明 |
|---|------|------|
| `Checker` | 構造体 | チェッカーオーケストレータ |
| `HttpHealthCheckOption` | 関数型 | `HttpHealthCheck` の設定オプション |
| `WithTimeout` | オプション関数 | HTTP チェックのタイムアウト設定 |
| `WithName` | オプション関数 | HTTP チェックの名前設定 |
| `PostgresHealthCheck` | 構造体 | PostgreSQL 接続確認 |
| `PostgresHealthCheckOption` | 関数型 | `PostgresHealthCheck` の設定オプション |
| `WithPostgresTimeout` | オプション関数 | PostgreSQL チェックのタイムアウト設定 |
| `WithPostgresName` | オプション関数 | PostgreSQL チェックの名前設定 |
| `RedisHealthCheck` | 構造体 | Redis PING 確認 |
| `RedisHealthCheckOption` | 関数型 | `RedisHealthCheck` の設定オプション |
| `WithRedisTimeout` | オプション関数 | Redis チェックのタイムアウト設定 |
| `WithRedisName` | オプション関数 | Redis チェックの名前設定 |

### TypeScript 固有

| 型 | 種別 | 説明 |
|---|------|------|
| `HealthChecker` | クラス | チェッカーオーケストレータ（Rust の `CompositeHealthChecker` 相当） |
| `HttpHealthCheckOptions` | type | `HttpHealthCheck` の設定オプション（`timeout?`, `name?` 等） |

### Rust 固有（追記）

| 型 | 種別 | 説明 |
|---|------|------|
| `MockHealthCheck` | 構造体（feature: `mock`） | テスト用モックヘルスチェッカー（`mockall::automock` による自動生成） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-health"
version = "0.1.0"
edition = "2021"

[features]
default = []
postgres = ["sqlx"]
redis = ["deadpool-redis"]
http = ["reqwest"]
kafka = ["rdkafka"]
mock = ["mockall"]
full = ["postgres", "redis", "http", "kafka"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
serde = { version = "1", features = ["derive"] }
mockall = { version = "0.13", optional = true }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"], optional = true }
deadpool-redis = { version = "0.18", optional = true }
reqwest = { version = "0.12", features = ["json"], optional = true }
rdkafka = { version = "0.36", features = ["cmake-build"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-health = { path = "../../system/library/rust/health", features = ["postgres", "redis"] }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
health/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── checker.rs      # HealthCheck トレイト, HealthChecker トレイト, CompositeHealthChecker
│   ├── error.rs        # HealthError enum
│   ├── checks/
│   │   ├── mod.rs      # feature-gated サブモジュール宣言
│   │   ├── http.rs     # HttpHealthCheck (feature: http)
│   │   ├── postgres.rs # PostgresHealthCheck (feature: postgres)
│   │   ├── redis.rs    # RedisHealthCheck (feature: redis)
│   │   └── kafka.rs    # KafkaHealthCheck (feature: kafka)
│   └── response.rs     # HealthStatus・CheckResult・HealthResponse・HealthzResponse
└── Cargo.toml
```

**使用例**:

```rust
use std::sync::Arc;
use k1s0_health::{
    CompositeHealthChecker, PostgresHealthCheck, RedisHealthCheck,
};

// CompositeHealthChecker の構築（mut が必要）
let mut checker = CompositeHealthChecker::new();
checker.add_check(Box::new(PostgresHealthCheck::new("database", pool.clone())));
checker.add_check(Box::new(RedisHealthCheck::new("redis", redis_pool.clone())));

let checker = Arc::new(checker);

// サーバー側で /healthz, /readyz ハンドラに組み込む（フレームワーク非依存）
// healthz: checker.healthz() -> HealthzResponse { status: "ok" }
// readyz:  checker.readyz().await -> HealthResponse
```

**レスポンス例** (`readyz`):

```json
{
  "status": "Healthy",
  "checks": {
    "database": { "status": "Healthy", "message": null },
    "redis":    { "status": "Healthy", "message": null }
  },
  "timestamp": "1709312400"
}
```

**主要 API**:

```rust
// HealthCheck トレイト -- 個別チェック実装インターフェース
#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> Result<(), HealthError>;
}

// HealthChecker トレイト
pub trait HealthChecker: Send + Sync {
    fn add_check(&mut self, check: Box<dyn HealthCheck>);
}

// CompositeHealthChecker -- 複数チェッカーの集約
impl CompositeHealthChecker {
    pub fn new() -> Self;
    pub fn add_check(&mut self, check: Box<dyn HealthCheck>);
    pub async fn run_all(&self) -> HealthResponse;
    pub async fn readyz(&self) -> HealthResponse;   // run_all のエイリアス
    pub fn healthz(&self) -> HealthzResponse;        // 常に { status: "ok" }
}

// HealthError
pub enum HealthError {
    CheckFailed(String),
    Timeout(String),
}

// HttpHealthCheck (feature = "http")
impl HttpHealthCheck {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self;
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self;  // デフォルト 5000ms
}

// PostgresHealthCheck (feature = "postgres")
impl PostgresHealthCheck {
    pub fn new(name: impl Into<String>, pool: PgPool) -> Self;
}

// RedisHealthCheck (feature = "redis")
impl RedisHealthCheck {
    pub fn new(name: impl Into<String>, pool: deadpool_redis::Pool) -> Self;
}

// KafkaHealthCheck (feature = "kafka")
impl KafkaHealthCheck {
    pub fn new(name: impl Into<String>, brokers: Vec<String>) -> Self;
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self;  // デフォルト 5000ms
}

// mock feature 有効時
pub use MockHealthCheck;  // mockall 自動生成
```

## Go 実装

**配置先**: `regions/system/library/go/health/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/redis/go-redis/v9`（`RedisHealthCheck` 使用時）。HTTP・PostgreSQL チェックは標準ライブラリのみ。

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
    Message string  // 成功時 "OK"、失敗時 err.Error()
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

// Checker -- オーケストレータ
type Checker struct{}

func NewChecker() *Checker
func (c *Checker) Add(check HealthCheck)
func (c *Checker) RunAll(ctx context.Context) HealthResponse
func (c *Checker) Readyz(ctx context.Context) HealthResponse  // RunAll のエイリアス
func (c *Checker) Healthz() map[string]string                 // {"status": "ok"}
```

**チェック実装の構築**:

```go
// HttpHealthCheck -- HTTP GET による外部ヘルスチェック
type HttpHealthCheckOption func(*HttpHealthCheck)

func NewHttpHealthCheck(url string, opts ...HttpHealthCheckOption) *HttpHealthCheck
func WithTimeout(d time.Duration) HttpHealthCheckOption    // デフォルト 5s
func WithName(name string) HttpHealthCheckOption           // デフォルト "http"

// PostgresHealthCheck -- PostgreSQL ping チェック
type PostgresHealthCheckOption func(*PostgresHealthCheck)

func NewPostgresHealthCheck(db *sql.DB, opts ...PostgresHealthCheckOption) *PostgresHealthCheck
func WithPostgresTimeout(d time.Duration) PostgresHealthCheckOption   // デフォルト 5s
func WithPostgresName(name string) PostgresHealthCheckOption          // デフォルト "postgres"

// RedisHealthCheck -- Redis PING チェック（要 github.com/redis/go-redis/v9）
type RedisHealthCheckOption func(*RedisHealthCheck)

func NewRedisHealthCheck(client redis.Cmdable, opts ...RedisHealthCheckOption) *RedisHealthCheck
func WithRedisTimeout(d time.Duration) RedisHealthCheckOption         // デフォルト 5s
func WithRedisName(name string) RedisHealthCheckOption                // デフォルト "redis"
```

**使用例**:

```go
checker := health.NewChecker()
checker.Add(health.NewHttpHealthCheck("https://api.example.com/healthz", health.WithName("api-gateway")))
checker.Add(health.NewPostgresHealthCheck(db, health.WithPostgresName("main-db")))
checker.Add(health.NewRedisHealthCheck(redisClient, health.WithRedisName("cache")))

// /readyz ハンドラ
resp := checker.Readyz(ctx)

// /healthz ハンドラ
liveness := checker.Healthz() // {"status": "ok"}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/health/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（native `fetch` API を使用）

**主要 API**:

```typescript
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy';

export interface CheckResult {
  status: HealthStatus;
  message?: string;
}

export interface HealthResponse {
  status: HealthStatus;
  checks: Record<string, CheckResult>;
  timestamp: string;  // ISO 8601 文字列
}

export interface HealthCheck {
  name: string;
  check(): Promise<void>;  // 成功: resolve、失敗: throw
}

export interface HttpHealthCheckOptions {
  url: string;
  timeoutMs?: number;  // デフォルト 5000
  name?: string;       // デフォルト "http"
}

export class HealthChecker {
  add(check: HealthCheck): void;
  async runAll(): Promise<HealthResponse>;
  async readyz(): Promise<HealthResponse>;     // runAll のエイリアス
  async healthz(): Promise<{ status: 'ok' }>;
}

export class HttpHealthCheck implements HealthCheck {
  readonly name: string;
  constructor(options: HttpHealthCheckOptions);
  check(): Promise<void>;
}
```

**使用例**:

```typescript
import { HealthChecker, HttpHealthCheck } from '@k1s0/health';

const checker = new HealthChecker();
checker.add(new HttpHealthCheck({ url: 'https://api.example.com/healthz', name: 'api-gateway' }));

// /readyz ハンドラ
const response = await checker.readyz();
// { status: 'healthy', checks: { 'api-gateway': { status: 'healthy' } }, timestamp: '2026-03-01T...' }

// /healthz ハンドラ
const liveness = await checker.healthz();
// { status: 'ok' }
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/health/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（`dart:io` `HttpClient` を使用）

**使用例**:

```dart
import 'package:k1s0_health/health.dart';

final checker = HealthChecker()
  ..add(HttpHealthCheck(name: 'api-gateway', url: 'https://api.example.com/healthz'));

final response = await checker.readyz();
print(response.status); // HealthStatus.healthy
```

**主要 API**:

```dart
enum HealthStatus { healthy, degraded, unhealthy }

class CheckResult {
  final HealthStatus status;
  final String? message;
  const CheckResult({required this.status, this.message});
}

class HealthResponse {
  final HealthStatus status;
  final Map<String, CheckResult> checks;
  final DateTime timestamp;
}

abstract class HealthCheck {
  String get name;
  Future<void> check();  // 成功: return、失敗: throw
}

class HealthChecker {
  void add(HealthCheck check);
  Future<HealthResponse> runAll();
  Future<HealthResponse> readyz();        // runAll のエイリアス
  Map<String, String> healthz();          // {'status': 'ok'}（同期）
}

class HttpHealthCheck implements HealthCheck {
  HttpHealthCheck({required String url, Duration timeout, String? name});
  // name デフォルト: 'http'、timeout デフォルト: Duration(seconds: 5)
}
```

**カバレッジ目標**: 90%以上

## テスト戦略

### Rust ユニットテスト (`cargo test --lib`)

`HealthCheck` トレイト実装のスタブを用意し、`CompositeHealthChecker` の集約ロジックを検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthy;
    struct AlwaysUnhealthy;

    #[async_trait]
    impl HealthCheck for AlwaysHealthy {
        fn name(&self) -> &str { "always-healthy" }
        async fn check(&self) -> Result<(), HealthError> { Ok(()) }
    }

    #[async_trait]
    impl HealthCheck for AlwaysUnhealthy {
        fn name(&self) -> &str { "always-unhealthy" }
        async fn check(&self) -> Result<(), HealthError> {
            Err(HealthError::CheckFailed("down".to_string()))
        }
    }

    #[tokio::test]
    async fn test_all_healthy() {
        let mut checker = CompositeHealthChecker::new();
        checker.add_check(Box::new(AlwaysHealthy));
        let resp = checker.readyz().await;
        assert_eq!(resp.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_one_unhealthy_degrades_overall() {
        let mut checker = CompositeHealthChecker::new();
        checker.add_check(Box::new(AlwaysHealthy));
        checker.add_check(Box::new(AlwaysUnhealthy));
        let resp = checker.readyz().await;
        assert_eq!(resp.status, HealthStatus::Unhealthy);
    }
}
```

### Go ユニットテスト (`go test ./...`)

```go
type alwaysHealthy struct{}
func (a alwaysHealthy) Name() string                        { return "always-healthy" }
func (a alwaysHealthy) Check(ctx context.Context) error     { return nil }

type alwaysUnhealthy struct{}
func (a alwaysUnhealthy) Name() string                      { return "always-unhealthy" }
func (a alwaysUnhealthy) Check(ctx context.Context) error   { return fmt.Errorf("down") }

func TestAllHealthy(t *testing.T) {
    c := health.NewChecker()
    c.Add(alwaysHealthy{})
    resp := c.RunAll(context.Background())
    assert.Equal(t, health.StatusHealthy, resp.Status)
}
```

### TypeScript ユニットテスト (vitest)

```typescript
const healthy: HealthCheck = { name: 'ok', check: async () => {} };
const unhealthy: HealthCheck = { name: 'bad', check: async () => { throw new Error('down'); } };

const checker = new HealthChecker();
checker.add(healthy);
const resp = await checker.readyz();
expect(resp.status).toBe('healthy');
```

### Dart ユニットテスト (test)

```dart
class AlwaysHealthy implements HealthCheck {
  @override String get name => 'ok';
  @override Future<void> check() async {}
}

final checker = HealthChecker()..add(AlwaysHealthy());
final resp = await checker.readyz();
expect(resp.status, equals(HealthStatus.healthy));
```

### 検証すべきケース（全言語共通）

- 全チェッカーが正常 -> レスポンス `status` が `Healthy`/`healthy` になること
- 1 つでも異常 -> レスポンス `status` が `Unhealthy`/`unhealthy` になること
- チェッカー未登録 -> レスポンス `status` が `Healthy`/`healthy`（空マップ）
- `readyz` が `runAll` と同等の結果を返すこと
- `healthz` が常に `{"status": "ok"}` を返すこと

> **注**: `Degraded` ステータスは全言語で定義されているが、現在のチェッカー実装ではいずれも自動的に `Degraded` を返すロジックを持たない。カスタム `HealthCheck` 実装で明示的に使用することは可能。

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-cache設計](../data/cache.md) — k1s0-cache ライブラリ
- [system-library-correlation設計](correlation.md) — k1s0-correlation ライブラリ
