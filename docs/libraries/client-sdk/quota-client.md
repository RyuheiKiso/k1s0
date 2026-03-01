# k1s0-quota-client ライブラリ設計

## 概要

quota-server へのクライアント SDK ライブラリ。`QuotaClient` トレイトによりクォータ残量の事前確認・使用量インクリメント・ポリシー取得を統一インターフェースで提供する。check before execute パターンによりリソース超過を事前に阻止し、TTL 付きキャッシュによるポリシーの高速参照を実現する。

クォータポリシーはローカルキャッシュで TTL 付きに保持し、quota-server への問い合わせ頻度を削減する。クォータ超過時は `QuotaExceededError` を返し、呼び出し側でリトライや代替処理への切り替えを判断できるよう設計する。

**配置先**: `regions/system/library/rust/quota-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `QuotaClient` | トレイト/インターフェース/抽象クラス | クォータ操作の抽象インターフェース（全4言語共通） |
| `HttpQuotaClient` | 構造体/クラス | quota-server HTTP REST 実装（全4言語） |
| `CachedQuotaClient` | 構造体/クラス | ポリシーキャッシュ付きラッパー（TTL 設定可、全4言語） |
| `InMemoryQuotaClient` | 構造体/クラス | テスト用インメモリ実装（Go/TypeScript/Dart） |
| `MockQuotaClient` | 構造体 | テスト用モック（Rust: feature = "mock" で有効） |
| `QuotaClientConfig` | 構造体/クラス | サーバー URL・タイムアウト・キャッシュ TTL 設定 |
| `QuotaStatus` | 構造体/クラス | 許可フラグ・残量・上限・リセット日時 |
| `QuotaUsage` | 構造体/クラス | クォータ ID・使用量・上限・期間・リセット日時 |
| `QuotaPolicy` | 構造体/クラス | クォータ ID・上限・期間・リセット戦略 |
| `QuotaPeriod` | enum | `Hourly` / `Daily` / `Monthly` / `Custom` |
| `QuotaClientError` | enum/クラス | 接続エラー・クォータ超過・NotFound 等 |

### QuotaClient インターフェース

全4言語において `QuotaClient` はトレイト・インターフェース・抽象クラスとして定義され、以下の4メソッドを提供する。

| メソッド | 引数 | 戻り値 | 説明 |
|---------|------|--------|------|
| `check` / `Check` | `quotaId: string`, `amount: uint64` | `QuotaStatus` | クォータ残量を事前確認（check before execute） |
| `increment` / `Increment` | `quotaId: string`, `amount: uint64` | `QuotaUsage` | クォータ使用量を加算 |
| `getUsage` / `GetUsage` | `quotaId: string` | `QuotaUsage` | 現在の使用量を取得 |
| `getPolicy` / `GetPolicy` | `quotaId: string` | `QuotaPolicy` | クォータポリシーを取得（CachedQuotaClient 経由でキャッシュ可） |

### クライアントの初期化

各実装クラスのコンストラクタおよびファクトリ関数：

| 言語 | コンストラクタ / ファクトリ | 引数 |
|------|--------------------------|------|
| Rust | `HttpQuotaClient::new(config)` | `QuotaClientConfig` |
| Rust | `CachedQuotaClient::new(inner, policy_ttl)` | `impl QuotaClient`, `Duration` |
| Go | `NewHttpQuotaClient(baseURL, config)` | `string`, `QuotaClientConfig` |
| Go | `NewQuotaClientConfig(baseURL)` | `string` |
| Go | `NewInMemoryQuotaClient()` | なし |
| Go | `NewCachedQuotaClient(inner, policyTTL)` | `QuotaClient`, `time.Duration` |
| TypeScript | `new HttpQuotaClient(config)` | `QuotaClientConfig` |
| TypeScript | `new CachedQuotaClient(inner, policyTtlMs)` | `QuotaClient`, `number` |
| Dart | `HttpQuotaClient(serverUrl, {httpClient?, timeout?, policyCacheTtl?})` | `String`, オプション引数 |
| Dart | `HttpQuotaClient.fromConfig(config, {httpClient?})` | `QuotaClientConfig` |
| Dart | `CachedQuotaClient(inner, policyTtl)` | `QuotaClient`, `Duration` |

### InMemoryQuotaClient

テスト用のインメモリ実装。Go/TypeScript/Dart で利用可能。ポリシーを事前に設定してからクォータ操作をテストするために使用する。

| メソッド | 言語 | 説明 |
|---------|------|------|
| `SetPolicy(quotaID, policy)` | Go | クォータ ID に対するポリシーを設定 |
| `setPolicy(quotaId, policy)` | TypeScript | クォータ ID に対するポリシーを設定 |
| `setPolicy(quotaId, policy)` | Dart | クォータ ID に対するポリシーを設定 |

**使用例（Dart）**:

```dart
final client = InMemoryQuotaClient();
client.setPolicy('storage:tenant-123', QuotaPolicy(
  quotaId: 'storage:tenant-123',
  limit: 1024 * 1024,
  period: QuotaPeriod.daily,
  resetStrategy: 'fixed',
));
final status = await client.check('storage:tenant-123', 512 * 1024);
assert(status.allowed);
```

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-quota-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
moka = { version = "0.12", features = ["future"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

**依存追加**: `k1s0-quota-client = { path = "../../system/library/rust/quota-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
quota-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # QuotaClient トレイト・HttpQuotaClient・CachedQuotaClient・MockQuotaClient
│   ├── config.rs       # QuotaClientConfig（サーバー URL・タイムアウト・TTL）
│   ├── model.rs        # QuotaStatus・QuotaUsage・QuotaPolicy・QuotaPeriod
│   └── error.rs        # QuotaClientError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_quota_client::{QuotaClient, QuotaClientConfig, HttpQuotaClient, CachedQuotaClient};
use std::time::Duration;

// HTTP クライアント + キャッシュラッパー
let config = QuotaClientConfig::new("http://quota-server:8080")
    .with_timeout(Duration::from_secs(5))
    .with_policy_cache_ttl(Duration::from_secs(60));

let http_client = HttpQuotaClient::new(config).unwrap();
let client = CachedQuotaClient::new(http_client, Duration::from_secs(60));

// check before execute パターン
let status = client.check("storage:tenant-123", 1024 * 1024).await.unwrap();
if !status.allowed {
    return Err(AppError::StorageQuotaExceeded);
}

// 操作実行後に使用量を記録
let usage = client.increment("storage:tenant-123", 1024 * 1024).await.unwrap();
println!("Used: {} / {} bytes", usage.used, usage.limit);

// 使用量の確認
let current = client.get_usage("storage:tenant-123").await.unwrap();
println!("Reset at: {}", current.reset_at);

// ポリシー取得（キャッシュ済み）
let policy = client.get_policy("storage:tenant-123").await.unwrap();
println!("Period: {:?}, Limit: {}", policy.period, policy.limit);
```

## Go 実装

**配置先**: `regions/system/library/go/quota-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.11.1`

**主要インターフェース**:

```go
type QuotaClient interface {
    Check(ctx context.Context, quotaID string, amount uint64) (*QuotaStatus, error)
    Increment(ctx context.Context, quotaID string, amount uint64) (*QuotaUsage, error)
    GetUsage(ctx context.Context, quotaID string) (*QuotaUsage, error)
    GetPolicy(ctx context.Context, quotaID string) (*QuotaPolicy, error)
}

type QuotaStatus struct {
    Allowed   bool
    Remaining uint64
    Limit     uint64
    ResetAt   time.Time
}

type QuotaUsage struct {
    QuotaID string
    Used    uint64
    Limit   uint64
    Period  QuotaPeriod
    ResetAt time.Time
}

type QuotaPeriod int

const (
    PeriodHourly  QuotaPeriod = iota
    PeriodDaily
    PeriodMonthly
    PeriodCustom  // 付加値なし（カスタム期間の ms 値は別途管理）
)

func NewInMemoryQuotaClient() *InMemoryQuotaClient
func (c *InMemoryQuotaClient) SetPolicy(quotaID string, policy *QuotaPolicy)
func NewCachedQuotaClient(inner QuotaClient, policyTTL time.Duration) *CachedQuotaClient
func NewQuotaClientConfig(baseURL string) QuotaClientConfig
func NewHttpQuotaClient(baseURL string, config QuotaClientConfig) *HttpQuotaClient
```

> **QuotaPeriod.Custom の言語別表現**:
>
> | 言語 | Custom バリアントの表現 | カスタム期間値 |
> |------|----------------------|--------------|
> | Rust | `QuotaPeriod::Custom(u64)` | タプルにミリ秒値を保持 |
> | TypeScript | `{ customMs: number }` | オブジェクトリテラル型でミリ秒値を保持 |
> | Go | `PeriodCustom`（定数） | 付加値なし |
> | Dart | `QuotaPeriod.custom`（enum 定数） | 付加値なし |

**使用例（HttpQuotaClient）**:

```go
config := NewQuotaClientConfig("http://quota-server:8080")
client := NewHttpQuotaClient("http://quota-server:8080", config)

// check before execute パターン
status, err := client.Check(ctx, "storage:tenant-123", 1024*1024)
if err != nil {
    return err
}
if !status.Allowed {
    return fmt.Errorf("quota exceeded")
}

// 操作実行後に使用量を記録
usage, err := client.Increment(ctx, "storage:tenant-123", 1024*1024)
if err != nil {
    return err
}
fmt.Printf("使用済み: %d / %d\n", usage.Used, usage.Limit)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/quota-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

> **QuotaClientConfig フィールド名の言語間マッピング**:
>
> | フィールド（設計上の名称） | Rust | Go | TypeScript | Dart |
> |--------------------------|------|----|------------|------|
> | `server_url` | `server_url` | `BaseURL` | `serverUrl` | `serverUrl` |
> | `timeout` | `timeout` | `Timeout` | `timeoutMs` ※ | `timeout` |
> | `policy_cache_ttl` | `policy_cache_ttl` | `PolicyCacheTTL` | `policyCacheTtlMs` ※ | `policyCacheTtl` |
>
> ※ TypeScript のみ `Ms` サフィックスを付与してミリ秒単位であることを名前に明示している。

**主要 API**:

```typescript
export interface QuotaStatus {
  allowed: boolean;
  remaining: number;
  limit: number;
  resetAt: Date;
}

export interface QuotaUsage {
  quotaId: string;
  used: number;
  limit: number;
  period: QuotaPeriod;
  resetAt: Date;
}

export interface QuotaPolicy {
  quotaId: string;
  limit: number;
  period: QuotaPeriod;
  resetStrategy: 'sliding' | 'fixed';
}

export type QuotaPeriod = 'hourly' | 'daily' | 'monthly' | { customMs: number };

export interface QuotaClient {
  check(quotaId: string, amount: number): Promise<QuotaStatus>;
  increment(quotaId: string, amount: number): Promise<QuotaUsage>;
  getUsage(quotaId: string): Promise<QuotaUsage>;
  getPolicy(quotaId: string): Promise<QuotaPolicy>;
}

export interface QuotaClientConfig {
  serverUrl: string;
  timeoutMs?: number;       // 他言語の `timeout` に相当（ミリ秒単位、デフォルト: 5000）
  policyCacheTtlMs?: number; // 他言語の `policy_cache_ttl` に相当（ミリ秒単位、デフォルト: 60000）
}

export class HttpQuotaClient implements QuotaClient {
  constructor(config: QuotaClientConfig);
  check(quotaId: string, amount: number): Promise<QuotaStatus>;
  increment(quotaId: string, amount: number): Promise<QuotaUsage>;
  getUsage(quotaId: string): Promise<QuotaUsage>;
  getPolicy(quotaId: string): Promise<QuotaPolicy>;
}

export class InMemoryQuotaClient implements QuotaClient {
  setPolicy(quotaId: string, policy: QuotaPolicy): void;
  check(quotaId: string, amount: number): Promise<QuotaStatus>;
  increment(quotaId: string, amount: number): Promise<QuotaUsage>;
  getUsage(quotaId: string): Promise<QuotaUsage>;
  getPolicy(quotaId: string): Promise<QuotaPolicy>;
}

export class CachedQuotaClient implements QuotaClient {
  constructor(inner: QuotaClient, policyTtlMs: number);
  check(quotaId: string, amount: number): Promise<QuotaStatus>;
  increment(quotaId: string, amount: number): Promise<QuotaUsage>;
  getUsage(quotaId: string): Promise<QuotaUsage>;
  getPolicy(quotaId: string): Promise<QuotaPolicy>;
}

export class QuotaExceededError extends Error {
  constructor(public readonly quotaId: string, public readonly remaining: number);
}

export class QuotaNotFoundError extends Error {
  constructor(public readonly quotaId: string);
}

export class QuotaConnectionError extends Error {
  constructor(message: string);
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/quota_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  http: ^1.2.0
  meta: ^1.14.0
```

**主要インターフェース**:

```dart
abstract class QuotaClient {
  Future<QuotaStatus> check(String quotaId, int amount);
  Future<QuotaUsage> increment(String quotaId, int amount);
  Future<QuotaUsage> getUsage(String quotaId);
  Future<QuotaPolicy> getPolicy(String quotaId);
}

class QuotaStatus {
  final bool allowed;
  final int remaining;
  final int limit;
  final DateTime resetAt;
}

class QuotaUsage {
  final String quotaId;
  final int used;
  final int limit;
  final QuotaPeriod period;
  final DateTime resetAt;
}

class QuotaPolicy {
  final String quotaId;
  final int limit;
  final QuotaPeriod period;
  final String resetStrategy;
}

enum QuotaPeriod { hourly, daily, monthly, custom }

class QuotaClientConfig {
  final String serverUrl;
  final Duration timeout;        // デフォルト: 5s
  final Duration policyCacheTtl; // デフォルト: 60s

  const QuotaClientConfig({
    required String serverUrl,
    Duration timeout = const Duration(seconds: 5),
    Duration policyCacheTtl = const Duration(seconds: 60),
  });
}

class HttpQuotaClient implements QuotaClient {
  HttpQuotaClient(String serverUrl, {http.Client? httpClient, Duration? timeout, Duration? policyCacheTtl});
  factory HttpQuotaClient.fromConfig(QuotaClientConfig config, {http.Client? httpClient});
  Future<QuotaStatus> check(String quotaId, int amount);
  Future<QuotaUsage> increment(String quotaId, int amount);
  Future<QuotaUsage> getUsage(String quotaId);
  Future<QuotaPolicy> getPolicy(String quotaId);
}

class InMemoryQuotaClient implements QuotaClient {
  void setPolicy(String quotaId, QuotaPolicy policy);
  Future<QuotaStatus> check(String quotaId, int amount);
  Future<QuotaUsage> increment(String quotaId, int amount);
  Future<QuotaUsage> getUsage(String quotaId);
  Future<QuotaPolicy> getPolicy(String quotaId);
}

class CachedQuotaClient implements QuotaClient {
  CachedQuotaClient(QuotaClient inner, Duration policyTtl);
  Future<QuotaStatus> check(String quotaId, int amount);
  Future<QuotaUsage> increment(String quotaId, int amount);
  Future<QuotaUsage> getUsage(String quotaId);
  Future<QuotaPolicy> getPolicy(String quotaId);
}

// エラー型
class QuotaClientError implements Exception {
  final String message;
  const QuotaClientError(this.message);
}

class QuotaExceededError extends QuotaClientError {
  final String quotaId;
  final int remaining;
  QuotaExceededError(this.quotaId, this.remaining)
      : super('Quota exceeded for $quotaId');
}

class QuotaNotFoundError extends QuotaClientError {
  final String quotaId;
  QuotaNotFoundError(this.quotaId)
      : super('Quota not found: $quotaId');
}

class QuotaConnectionError extends QuotaClientError {
  QuotaConnectionError(String message) : super('Connection error: $message');
}
```

**カバレッジ目標**: 85%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | check/increment ロジック・キャッシュ TTL 判定・エラーハンドリング | tokio::test |
| モックテスト | `mockall` による QuotaClient モック・クォータ超過シナリオ | mockall (feature = "mock") |
| キャッシュテスト | TTL 期限切れでの再取得・TTL 内でのキャッシュヒット検証 | tokio::test + 時間操作 |
| 統合テスト | wiremock による quota-server レスポンスシミュレーション | wiremock |
| プロパティテスト | 任意クォータ ID・使用量での check/increment ラウンドトリップ | proptest |

## 通知連携

クォータ超過時の通知フローは以下の設計に基づく。

1. **quota-server** がクォータ超過を検知すると `k1s0.system.quota.exceeded.v1` トピックにイベントを発行する
2. **notification-server** は `k1s0.system.notification.requested.v1` トピックを購読して通知を送信する
3. この 2 つのトピック間には変換レイヤーが必要である。quota-server が発行する `k1s0.system.quota.exceeded.v1` イベントを `k1s0.system.notification.requested.v1` 形式に変換して再発行するコンシューマー（またはサーバー側のイベントハンドラー）を設ける

> **注意**: quota-client 自体は通知送信の責務を持たない。クォータ超過イベントの発行は quota-server 側の責務であり、通知への変換はサーバー間連携で行う。

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-file-client設計](file-client.md) — ストレージクォータ管理の利用例
- [system-library-cache設計](../data/cache.md) — ポリシーキャッシュの実装参考
- [system-ratelimit-server設計](../../servers/ratelimit/server.md) — レートリミットとクォータの関係
