# k1s0-featureflag ライブラリ設計

## 概要

フィーチャーフラグサーバーのクライアント SDK。`FeatureFlagClient` トレイト/インターフェースにより `evaluate`（フラグ評価）、`get_flag`/`getFlag`（フラグ詳細取得）、`is_enabled`/`isEnabled`（有効/無効判定）を提供する。現在は `InMemoryFeatureFlagClient` を実装しており、テストやローカル開発に使用する。

## 公開 API（全言語共通契約）

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `FeatureFlagClient` | トレイト/インターフェース | フラグ評価の抽象インターフェース（evaluate, get_flag, is_enabled） |
| `InMemoryFeatureFlagClient` | 構造体/クラス | テスト用インメモリ実装 |
| `EvaluationContext` | 構造体/クラス | 評価コンテキスト（user_id, tenant_id, attributes） |
| `EvaluationResult` | 構造体/クラス | 評価結果（flag_key, enabled, variant, reason） |
| `FeatureFlag` | 構造体/クラス | フラグ定義（id, flag_key, description, enabled, variants） |
| `FlagVariant` | 構造体/クラス | バリアント定義（name, value, weight） |
| `FeatureFlagError` | enum/クラス | フラグ未定義エラー・接続エラー等 |
| `MockFeatureFlagClient` | 構造体 | Rust テスト用モック（feature = "mock" で有効） |

## Rust 実装

**配置先**: `regions/system/library/rust/featureflag/`

**Cargo.toml**:

```toml
[package]
name = "k1s0-featureflag"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
tracing = "0.1"
mockall = { version = "0.13", optional = true }
```

**依存追加**: `k1s0-featureflag = { path = "../../system/library/rust/featureflag" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
featureflag/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # FeatureFlagClient トレイト + EvaluationResult
│   ├── context.rs      # EvaluationContext
│   ├── flag.rs         # FeatureFlag + FlagVariant
│   ├── error.rs        # FeatureFlagError
│   └── memory.rs       # InMemoryFeatureFlagClient
├── tests/
│   └── featureflag_test.rs
└── Cargo.toml
```

**主要 API**:

```rust
use async_trait::async_trait;

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait FeatureFlagClient: Send + Sync {
    async fn evaluate(&self, flag_key: &str, context: &EvaluationContext) -> Result<EvaluationResult, FeatureFlagError>;
    async fn get_flag(&self, flag_key: &str) -> Result<FeatureFlag, FeatureFlagError>;
    async fn is_enabled(&self, flag_key: &str, context: &EvaluationContext) -> Result<bool, FeatureFlagError>;
}

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl EvaluationContext {
    pub fn new() -> Self { /* Default */ }
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self { /* ... */ }
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self { /* ... */ }
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self { /* ... */ }
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub flag_key: String,
    pub enabled: bool,
    pub variant: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagVariant {
    pub name: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Error)]
pub enum FeatureFlagError {
    #[error("フラグが見つかりません: {key}")]
    FlagNotFound { key: String },
    #[error("接続エラー: {0}")]
    ConnectionError(String),
    #[error("設定エラー: {0}")]
    ConfigError(String),
}
```

**使用例**:

```rust
use k1s0_featureflag::{InMemoryFeatureFlagClient, FeatureFlagClient, EvaluationContext, FeatureFlag, FlagVariant};

let client = InMemoryFeatureFlagClient::new();

// フラグを登録
client.set_flag(FeatureFlag {
    id: "flag-1".into(),
    flag_key: "new-checkout-flow".into(),
    description: "新しいチェックアウトフロー".into(),
    enabled: true,
    variants: vec![],
}).await;

// フラグ評価
let ctx = EvaluationContext::new()
    .with_user_id("user-uuid-1234")
    .with_tenant_id("TENANT-001");

let result = client.evaluate("new-checkout-flow", &ctx).await.unwrap();
if result.enabled {
    // 新しいチェックアウトフローを使用
}

// 簡易判定
let enabled = client.is_enabled("new-checkout-flow", &ctx).await.unwrap();

// フラグ詳細取得
let flag = client.get_flag("new-checkout-flow").await.unwrap();
println!("description: {}", flag.description);
```

**テスト例**:

> 主要テスト: `test_evaluate_enabled_flag`, `test_get_flag_by_key`, `test_is_enabled_true`, `test_mock_feature_flag_client`。全10件の詳細は `tests/featureflag_test.rs` を参照。

**カバレッジ目標**: 90%以上

## Go 実装

**配置先**: `regions/system/library/go/featureflag/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.11`（テスト用）

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

**エラー型**:

```go
type FeatureFlagError struct {
    Code    string
    Message string
}

func (e *FeatureFlagError) Error() string { return fmt.Sprintf("%s: %s", e.Code, e.Message) }
func NewFlagNotFoundError(key string) *FeatureFlagError { /* ... */ }
func NewConnectionError(msg string) *FeatureFlagError { /* ... */ }
```

**InMemoryFeatureFlagClient**:

```go
func NewInMemoryFeatureFlagClient() *InMemoryFeatureFlagClient
func (c *InMemoryFeatureFlagClient) SetFlag(flag *FeatureFlag)
// Evaluate, GetFlag, IsEnabled を実装
```

**テスト例**:

> 主要テスト: `TestEvaluate_EnabledFlag`, `TestGetFlag`, `TestIsEnabled`, `TestEvaluationContext`。全7件の詳細は `featureflag_test.go` を参照。

**カバレッジ目標**: 90%以上

## TypeScript 実装

**配置先**: `regions/system/library/typescript/featureflag/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface FlagVariant {
  name: string;
  value: string;
  weight: number;
}

export interface FeatureFlag {
  id: string;
  flagKey: string;
  description: string;
  enabled: boolean;
  variants: FlagVariant[];
}

export interface EvaluationContext {
  userId?: string;
  tenantId?: string;
  attributes?: Record<string, string>;
}

export interface EvaluationResult {
  flagKey: string;
  enabled: boolean;
  variant?: string;
  reason: string;
}

export interface FeatureFlagClient {
  evaluate(flagKey: string, context: EvaluationContext): Promise<EvaluationResult>;
  getFlag(flagKey: string): Promise<FeatureFlag>;
  isEnabled(flagKey: string, context: EvaluationContext): Promise<boolean>;
}

export class FeatureFlagError extends Error {
  constructor(message: string, public readonly code: string) { super(message); }
}

export class InMemoryFeatureFlagClient implements FeatureFlagClient {
  setFlag(flag: FeatureFlag): void;
  evaluate(flagKey: string, context: EvaluationContext): Promise<EvaluationResult>;
  getFlag(flagKey: string): Promise<FeatureFlag>;
  isEnabled(flagKey: string, context: EvaluationContext): Promise<boolean>;
}
```

**テスト例**:

> 主要テスト: `有効フラグのevaluateでenabled=trueを返す`, `getFlagでフラグ情報を取得できる`, `isEnabledで有効フラグはtrueを返す`。全9件の詳細は `__tests__/featureflag.test.ts` を参照。

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/featureflag/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
class FlagVariant {
  final String name;
  final String value;
  final double weight;
  const FlagVariant({required this.name, required this.value, this.weight = 1.0});
}

class FeatureFlag {
  final String id;
  final String flagKey;
  final String description;
  final bool enabled;
  final List<FlagVariant> variants;
  const FeatureFlag({required this.id, required this.flagKey, required this.description, required this.enabled, this.variants = const []});
}

class EvaluationContext {
  final String? userId;
  final String? tenantId;
  final Map<String, String> attributes;
  const EvaluationContext({this.userId, this.tenantId, this.attributes = const {}});
}

class EvaluationResult {
  final bool enabled;
  final String? variant;
  final String reason;
  const EvaluationResult({required this.enabled, this.variant, required this.reason});
}

abstract class FeatureFlagClient {
  Future<EvaluationResult> evaluate(String flagKey, EvaluationContext context);
  Future<bool> isEnabled(String flagKey, EvaluationContext context);
  Future<FeatureFlag?> getFlag(String flagKey);
}

class FeatureFlagNotFoundException implements Exception {
  final String flagKey;
  const FeatureFlagNotFoundException(this.flagKey);
}

class InMemoryFeatureFlagClient implements FeatureFlagClient {
  void setFlag(FeatureFlag flag);
  // evaluate, isEnabled, getFlag を実装
}
```

**テスト例**:

> 主要テスト: `evaluate enabled flag returns enabled`, `getFlag returns flag when it exists`, `isEnabled returns true for enabled flag`。全9件の詳細は `test/featureflag_test.dart` を参照。

**カバレッジ目標**: 90%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-featureflag-server設計](../../servers/featureflag/server.md) — サーバー設計
- [system-library-serviceauth設計](../auth-security/serviceauth.md) — gRPC 認証パターン
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [可観測性設計](../../architecture/observability/可観測性設計.md) — メトリクス・トレーシング設計
