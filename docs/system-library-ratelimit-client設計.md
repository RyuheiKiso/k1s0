# k1s0-ratelimit-client ライブラリ設計

## 概要

system-ratelimit-server（ポート 8080）へのレート制限クライアントライブラリ。レート制限の事前確認（check before execute パターン）・使用量消費の記録・制限超過時の待機時間返却・テナントや API キーごとの制限照会を統一インターフェースで提供する。全 Tier のサービスから共通利用し、API ゲートウェイ・バックエンドサービス両方で一貫したレート制限を実現する。

**配置先**: `regions/system/library/rust/ratelimit-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `RateLimitClient` | トレイト | レート制限操作インターフェース |
| `GrpcRateLimitClient` | 構造体 | gRPC 経由の ratelimit-server 接続実装 |
| `RateLimitStatus` | 構造体 | 許可フラグ・残余カウント・リセット時刻・再試行待機秒数 |
| `RateLimitResult` | 構造体 | 消費後の残余カウント・リセット時刻 |
| `RateLimitPolicy` | 構造体 | キーに紐づく制限設定（上限・ウィンドウ・アルゴリズム）|
| `RateLimitError` | enum | `LimitExceeded`・`KeyNotFound`・`ServerError`・`Timeout` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-ratelimit-client"
version = "0.1.0"
edition = "2021"

[features]
grpc = ["tonic"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
tonic = { version = "0.12", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-ratelimit-client = { path = "../../system/library/rust/ratelimit-client" }
# gRPC 経由を有効化する場合:
k1s0-ratelimit-client = { path = "../../system/library/rust/ratelimit-client", features = ["grpc"] }
```

**モジュール構成**:

```
ratelimit-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # RateLimitClient トレイト
│   ├── grpc.rs         # GrpcRateLimitClient
│   ├── types.rs        # RateLimitStatus・RateLimitResult・RateLimitPolicy
│   └── error.rs        # RateLimitError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_ratelimit_client::{GrpcRateLimitClient, RateLimitClient};

// クライアントの構築
let client = GrpcRateLimitClient::new("http://ratelimit-server:8080").await?;

// check before execute パターン
let key = "tenant:TENANT-001:api:/v1/orders";
let status = client.check(key, 1).await?;

if !status.allowed {
    let retry_after = status.retry_after_secs.unwrap_or(60);
    return Err(format!("Rate limit exceeded. Retry after {}s", retry_after));
}

// 処理を実行してから使用量を消費
let result = execute_business_logic().await?;
client.consume(key, 1).await?;

// テナント単位の制限ポリシーを照会
let policy = client.get_limit("tenant:TENANT-001").await?;
tracing::info!(
    limit = policy.limit,
    window_secs = policy.window_secs,
    algorithm = %policy.algorithm,
    "レート制限ポリシー取得"
);
```

## Go 実装

**配置先**: `regions/system/library/go/ratelimit-client/`

```
ratelimit-client/
├── ratelimit_client.go
├── grpc_client.go
├── types.go
├── ratelimit_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `google.golang.org/grpc v1.70`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type RateLimitClient interface {
    Check(ctx context.Context, key string, cost uint32) (RateLimitStatus, error)
    Consume(ctx context.Context, key string, cost uint32) (RateLimitResult, error)
    GetLimit(ctx context.Context, key string) (RateLimitPolicy, error)
}

type RateLimitStatus struct {
    Allowed         bool
    Remaining       uint32
    ResetAt         time.Time
    RetryAfterSecs  *uint64
}

type RateLimitResult struct {
    Remaining uint32
    ResetAt   time.Time
}

type RateLimitPolicy struct {
    Key        string
    Limit      uint32
    WindowSecs uint64
    Algorithm  string
}

type GrpcRateLimitClient struct{ /* ... */ }

func NewGrpcRateLimitClient(addr string) (*GrpcRateLimitClient, error)
func (c *GrpcRateLimitClient) Check(ctx context.Context, key string, cost uint32) (RateLimitStatus, error)
func (c *GrpcRateLimitClient) Consume(ctx context.Context, key string, cost uint32) (RateLimitResult, error)
func (c *GrpcRateLimitClient) GetLimit(ctx context.Context, key string) (RateLimitPolicy, error)
```

**使用例**:

```go
client, err := NewGrpcRateLimitClient("ratelimit-server:8080")
if err != nil {
    log.Fatal(err)
}

key := "tenant:TENANT-001:api:/v1/orders"
status, err := client.Check(ctx, key, 1)
if err != nil {
    return err
}
if !status.Allowed {
    return fmt.Errorf("rate limit exceeded, retry after %d seconds", *status.RetryAfterSecs)
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/ratelimit-client/`

```
ratelimit-client/
├── package.json        # "@k1s0/ratelimit-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # RateLimitClient, GrpcRateLimitClient, RateLimitStatus, RateLimitResult, RateLimitPolicy, RateLimitError
└── __tests__/
    └── ratelimit-client.test.ts
```

**主要 API**:

```typescript
export interface RateLimitClient {
  check(key: string, cost: number): Promise<RateLimitStatus>;
  consume(key: string, cost: number): Promise<RateLimitResult>;
  getLimit(key: string): Promise<RateLimitPolicy>;
}

export interface RateLimitStatus {
  allowed: boolean;
  remaining: number;
  resetAt: Date;
  retryAfterSecs?: number;
}

export interface RateLimitResult {
  remaining: number;
  resetAt: Date;
}

export interface RateLimitPolicy {
  key: string;
  limit: number;
  windowSecs: number;
  algorithm: 'token_bucket' | 'sliding_window' | 'fixed_window';
}

export class GrpcRateLimitClient implements RateLimitClient {
  constructor(serverUrl: string);
  check(key: string, cost: number): Promise<RateLimitStatus>;
  consume(key: string, cost: number): Promise<RateLimitResult>;
  getLimit(key: string): Promise<RateLimitPolicy>;
  close(): Promise<void>;
}

export class RateLimitError extends Error {
  constructor(
    message: string,
    public readonly code: 'LIMIT_EXCEEDED' | 'KEY_NOT_FOUND' | 'SERVER_ERROR' | 'TIMEOUT',
    public readonly retryAfterSecs?: number
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/ratelimit_client/`

```
ratelimit_client/
├── pubspec.yaml        # k1s0_ratelimit_client
├── analysis_options.yaml
├── lib/
│   ├── ratelimit_client.dart
│   └── src/
│       ├── client.dart         # RateLimitClient abstract, GrpcRateLimitClient
│       ├── types.dart          # RateLimitStatus, RateLimitResult, RateLimitPolicy
│       └── error.dart          # RateLimitError
└── test/
    └── ratelimit_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  grpc: ^4.0.0
  protobuf: ^3.1.0
```

**使用例**:

```dart
import 'package:k1s0_ratelimit_client/ratelimit_client.dart';

final client = GrpcRateLimitClient('ratelimit-server:8080');

final key = 'tenant:TENANT-001:api:/v1/orders';
final status = await client.check(key, 1);

if (!status.allowed) {
  final retryAfter = status.retryAfterSecs ?? 60;
  throw RateLimitError('Rate limit exceeded. Retry after ${retryAfter}s');
}

await client.consume(key, 1);
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_status_allowed() {
        let status = RateLimitStatus {
            allowed: true,
            remaining: 99,
            reset_at: Utc::now() + chrono::Duration::seconds(60),
            retry_after_secs: None,
        };
        assert!(status.allowed);
        assert_eq!(status.remaining, 99);
        assert!(status.retry_after_secs.is_none());
    }

    #[test]
    fn test_rate_limit_status_denied() {
        let status = RateLimitStatus {
            allowed: false,
            remaining: 0,
            reset_at: Utc::now() + chrono::Duration::seconds(30),
            retry_after_secs: Some(30),
        };
        assert!(!status.allowed);
        assert_eq!(status.retry_after_secs, Some(30));
    }

    #[test]
    fn test_limit_exceeded_error() {
        let err = RateLimitError::LimitExceeded { retry_after_secs: 42 };
        assert!(matches!(err, RateLimitError::LimitExceeded { .. }));
    }
}
```

### 統合テスト

- `testcontainers` で ratelimit-server コンテナを起動して実際の check/consume フローを検証
- check で allowed=false が返るシナリオ（連続リクエストによる超過）を確認
- `retry_after_secs` が正しく返却されることを確認
- `get_limit` でポリシーが取得できることを確認

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestRateLimitClient {}
    #[async_trait]
    impl RateLimitClient for TestRateLimitClient {
        async fn check(&self, key: &str, cost: u32) -> Result<RateLimitStatus, RateLimitError>;
        async fn consume(&self, key: &str, cost: u32) -> Result<RateLimitResult, RateLimitError>;
        async fn get_limit(&self, key: &str) -> Result<RateLimitPolicy, RateLimitError>;
    }
}

#[tokio::test]
async fn test_api_handler_rejects_over_limit_request() {
    let mut mock = MockTestRateLimitClient::new();
    mock.expect_check()
        .once()
        .returning(|_, _| Ok(RateLimitStatus {
            allowed: false,
            remaining: 0,
            reset_at: Utc::now(),
            retry_after_secs: Some(10),
        }));

    let handler = ApiHandler::new(Arc::new(mock));
    let result = handler.handle_request("test-key").await;
    assert!(result.is_err());
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-ratelimit-server設計](system-ratelimit-server設計.md) — レート制限サーバー設計
- [system-library-tenant-client設計](system-library-tenant-client設計.md) — テナントクライアント（テナント ID キー生成）
- [system-library-cache設計](system-library-cache設計.md) — k1s0-cache ライブラリ（ローカルキャッシュ）
- [system-library-circuit-breaker設計](system-library-circuit-breaker設計.md) — サーキットブレーカー
