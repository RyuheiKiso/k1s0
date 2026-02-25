# k1s0-featureflag ライブラリ設計

## 概要

フィーチャーフラグサーバーのクライアント SDK。gRPC でフラグ値を取得し、ローカルキャッシュと Kafka Subscribe によるリアルタイム更新を組み合わせて低レイテンシな評価を実現する。`FeatureFlagClient` トレイトにより `evaluate`（有効/無効判定）と `get`（フラグ詳細取得）を提供する。

**配置先**: `regions/system/library/rust/featureflag/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `FeatureFlagClient` | トレイト | フラグ評価の抽象インターフェース |
| `GrpcFeatureFlagClient` | 構造体 | featureflag-server への gRPC 実装（moka キャッシュ + Kafka 更新） |
| `MockFeatureFlagClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `FlagEvaluationContext` | 構造体 | 評価コンテキスト（environment, user_id, service_name） |
| `FeatureFlag` | 構造体 | フラグ定義（key, enabled, rollout_percentage, target_environments） |
| `FeatureFlagConfig` | 構造体 | gRPC エンドポイント・キャッシュ TTL・Kafka 設定 |
| `FeatureFlagError` | enum | gRPC エラー・フラグ未定義エラー等 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-featureflag"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]
kafka = ["rdkafka"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
tonic = "0.12"
moka = { version = "0.12", features = ["future"] }
rdkafka = { version = "0.36", optional = true }
mockall = { version = "0.13", optional = true }
```

**Cargo.toml への追加行**:

```toml
k1s0-featureflag = { path = "../../system/library/rust/featureflag" }
# Kafka リアルタイム更新を有効化する場合:
k1s0-featureflag = { path = "../../system/library/rust/featureflag", features = ["kafka"] }
# テスト時にモックを有効化する場合:
k1s0-featureflag = { path = "../../system/library/rust/featureflag", features = ["mock"] }
```

**モジュール構成**:

```
featureflag/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # FeatureFlagClient トレイト・GrpcFeatureFlagClient・MockFeatureFlagClient
│   ├── context.rs      # FlagEvaluationContext
│   ├── config.rs       # FeatureFlagConfig（gRPC エンドポイント・TTL）
│   ├── cache.rs        # moka キャッシュ管理
│   ├── subscriber.rs   # Kafka トピック購読による自動キャッシュ更新
│   └── error.rs        # FeatureFlagError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_featureflag::{GrpcFeatureFlagClient, FeatureFlagClient, FlagEvaluationContext, FeatureFlagConfig};
use std::time::Duration;

let config = FeatureFlagConfig::new("http://featureflag-service:50051")
    .with_cache_ttl(Duration::from_secs(60))
    .with_environment("production");

let client = GrpcFeatureFlagClient::new(config).await.unwrap();

// フラグ評価
let ctx = FlagEvaluationContext::new("production")
    .with_user_id("user-uuid-1234")
    .with_service_name("order-service");

let enabled = client.evaluate("new-checkout-flow", &ctx).await.unwrap();

if enabled {
    // 新しいチェックアウトフローを使用
} else {
    // 従来のチェックアウトフローを使用
}

// フラグ詳細取得
let flag = client.get("new-checkout-flow").await.unwrap();
println!("rollout: {}%", flag.rollout_percentage);
```

## Go 実装

**配置先**: `regions/system/library/go/featureflag/`

```
featureflag/
├── featureflag.go
├── client.go
├── context.go
├── cache.go
├── featureflag_test.go
├── go.mod
└── go.sum
```

**依存関係**: `google.golang.org/grpc v1.71`, `github.com/patrickmn/go-cache v2.1`

**主要インターフェース**:

```go
type FeatureFlagClient interface {
    Evaluate(ctx context.Context, flagKey string, evalCtx *EvaluationContext) (*EvaluationResult, error)
    GetFlag(ctx context.Context, flagKey string) (*FeatureFlag, error)
    IsEnabled(ctx context.Context, flagKey string, evalCtx *EvaluationContext) (bool, error)
}

type EvaluationContext struct {
    UserID     *string
    TenantID   *string
    Attributes map[string]string
}

func NewEvaluationContext() *EvaluationContext
func (c *EvaluationContext) WithUserID(userID string) *EvaluationContext
func (c *EvaluationContext) WithTenantID(tenantID string) *EvaluationContext
func (c *EvaluationContext) WithAttribute(key, value string) *EvaluationContext

type EvaluationResult struct {
    FlagKey string
    Enabled bool
    Variant *string
    Reason  string
}

type FlagVariant struct {
    Name   string
    Value  string
    Weight int
}

type FeatureFlag struct {
    ID          string
    FlagKey     string
    Description string
    Enabled     bool
    Variants    []FlagVariant
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/featureflag/`

```
featureflag/
├── package.json        # "@k1s0/featureflag", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # FeatureFlagClient, EvaluationContext, FeatureFlag, FeatureFlagConfig
└── __tests__/
    ├── client.test.ts
    └── cache.test.ts
```

**主要 API**:

```typescript
export interface FeatureFlagClient {
  evaluate(key: string, context: EvaluationContext): Promise<boolean>;
  getFlag(key: string): Promise<FeatureFlag | null>;
  close(): void;
}

export interface EvaluationContext {
  environment: string;
  userId?: string;
  serviceName?: string;
}

export interface FeatureFlag {
  key: string;
  enabled: boolean;
  rolloutPercentage: number;
  targetEnvironments: string[];
  updatedAt: string;
}

export interface FeatureFlagConfig {
  endpoint: string;
  cacheTtlMs?: number;
  environment?: string;
}

export function createFeatureFlagClient(config: FeatureFlagConfig): FeatureFlagClient;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/featureflag/`

```
featureflag/
├── pubspec.yaml        # k1s0_featureflag, grpc: ^4.0.0, protobuf: ^3.1.0
├── analysis_options.yaml
├── lib/
│   ├── featureflag.dart
│   └── src/
│       ├── client.dart     # FeatureFlagClient abstract, GrpcFeatureFlagClient
│       ├── context.dart    # EvaluationContext
│       ├── config.dart     # FeatureFlagConfig
│       ├── cache.dart      # ローカルキャッシュ管理
│       ├── models.dart     # FeatureFlag
│       └── error.dart      # FeatureFlagError
└── test/
    ├── client_test.dart
    └── cache_test.dart
```

**カバレッジ目標**: 90%以上

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-featureflag-server設計](system-featureflag-server設計.md) — サーバー設計
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — gRPC 認証パターン
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [可観測性設計](可観測性設計.md) — メトリクス・トレーシング設計
