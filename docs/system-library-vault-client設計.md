# k1s0-vault-client ライブラリ設計

## 概要

system-vault-server（ポート 8091）へのシークレット取得・管理クライアントライブラリ。シークレット値の取得・メモリ内 TTL 付きキャッシュ・動的クレデンシャルのリース管理（自動更新）・シークレット更新の検知（Kafka トピック `k1s0.system.vault.secret_rotated.v1` のポーリングまたは購読）・複数シークレットの一括取得を提供する。全 Tier のサービスがシークレット管理を安全かつ効率的に行うための基盤ライブラリである。

**配置先**: `regions/system/library/rust/vault-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `VaultClient` | トレイト | シークレット操作インターフェース |
| `GrpcVaultClient` | 構造体 | gRPC 経由の vault-server 接続実装（TTL キャッシュ・ウォッチャー内蔵）|
| `Secret` | 構造体 | パス・データマップ・バージョン・作成日時 |
| `SecretWatcher` | 構造体 | シークレット更新通知の受信ハンドル（`Stream<SecretRotatedEvent>`）|
| `SecretRotatedEvent` | 構造体 | ローテーション通知（パス・新バージョン）|
| `VaultClientConfig` | 構造体 | サーバー URL・キャッシュ TTL・最大キャッシュサイズ・Kafka ブローカー |
| `VaultError` | enum | `NotFound`・`PermissionDenied`・`ServerError`・`Timeout`・`LeaseExpired` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-vault-client"
version = "0.1.0"
edition = "2021"

[features]
grpc = ["tonic"]
kafka = ["rdkafka"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
moka = { version = "0.12", features = ["future"] }
tokio = { version = "1", features = ["sync", "time"] }
tokio-stream = "0.1"
tonic = { version = "0.12", optional = true }
rdkafka = { version = "0.37", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-vault-client = { path = "../../system/library/rust/vault-client" }
# gRPC + Kafka ウォッチを有効化する場合:
k1s0-vault-client = { path = "../../system/library/rust/vault-client", features = ["grpc", "kafka"] }
```

**モジュール構成**:

```
vault-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # VaultClient トレイト
│   ├── grpc.rs         # GrpcVaultClient（TTL キャッシュ・ウォッチャー内蔵）
│   ├── secret.rs       # Secret・SecretWatcher・SecretRotatedEvent
│   ├── config.rs       # VaultClientConfig
│   └── error.rs        # VaultError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_vault_client::{GrpcVaultClient, VaultClient, VaultClientConfig};
use std::time::Duration;

// クライアントの構築（TTL 10 分・最大 500 件キャッシュ）
let config = VaultClientConfig::new("http://vault-server:8080")
    .cache_ttl(Duration::from_secs(600))
    .cache_max_capacity(500);

let client = GrpcVaultClient::new(config).await?;

// シークレット全体の取得（キャッシュ優先）
let secret = client.get_secret("system/database/primary").await?;
let db_password = secret.data.get("password").expect("password key missing");

// 特定キーの値を直接取得
let api_key = client.get_secret_value("system/external-api/stripe", "secret_key").await?;

// パスプレフィックスでシークレット一覧を取得
let paths = client.list_secrets("system/").await?;
for path in &paths {
    tracing::info!(path = %path, "シークレットパス");
}

// シークレット更新の監視（ローテーション時に自動で再取得）
let mut watcher = client.watch_secret("system/database/primary").await?;
tokio::spawn(async move {
    while let Some(event) = watcher.next().await {
        tracing::info!(
            path = %event.path,
            new_version = event.version,
            "シークレットがローテーションされました"
        );
        // アプリケーションのコネクションプールを再初期化するなどの処理
    }
});
```

## Go 実装

**配置先**: `regions/system/library/go/vault-client/`

```
vault-client/
├── vault_client.go
├── grpc_client.go
├── secret.go
├── config.go
├── vault_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `google.golang.org/grpc v1.70`, `github.com/segmentio/kafka-go v0.4`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type VaultClient interface {
    GetSecret(ctx context.Context, path string) (Secret, error)
    GetSecretValue(ctx context.Context, path, key string) (string, error)
    ListSecrets(ctx context.Context, pathPrefix string) ([]string, error)
    WatchSecret(ctx context.Context, path string) (<-chan SecretRotatedEvent, error)
}

type Secret struct {
    Path      string            `json:"path"`
    Data      map[string]string `json:"data"`
    Version   int64             `json:"version"`
    CreatedAt time.Time         `json:"created_at"`
}

type SecretRotatedEvent struct {
    Path    string `json:"path"`
    Version int64  `json:"version"`
}

type VaultClientConfig struct {
    ServerURL        string
    CacheTTL         time.Duration
    CacheMaxCapacity int
    KafkaBrokers     string
}

type GrpcVaultClient struct{ /* ... */ }

func NewGrpcVaultClient(config VaultClientConfig) (*GrpcVaultClient, error)
func (c *GrpcVaultClient) GetSecret(ctx context.Context, path string) (Secret, error)
func (c *GrpcVaultClient) GetSecretValue(ctx context.Context, path, key string) (string, error)
func (c *GrpcVaultClient) ListSecrets(ctx context.Context, pathPrefix string) ([]string, error)
func (c *GrpcVaultClient) WatchSecret(ctx context.Context, path string) (<-chan SecretRotatedEvent, error)
```

**使用例**:

```go
config := VaultClientConfig{
    ServerURL: "http://vault-server:8080",
    CacheTTL:  10 * time.Minute,
}
client, err := NewGrpcVaultClient(config)
if err != nil {
    log.Fatal(err)
}

// DB パスワードの取得
dbPassword, err := client.GetSecretValue(ctx, "system/database/primary", "password")
if err != nil {
    return err
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/vault-client/`

```
vault-client/
├── package.json        # "@k1s0/vault-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # VaultClient, GrpcVaultClient, Secret, SecretWatcher, SecretRotatedEvent, VaultClientConfig, VaultError
└── __tests__/
    └── vault-client.test.ts
```

**主要 API**:

```typescript
export interface Secret {
  path: string;
  data: Record<string, string>;
  version: number;
  createdAt: Date;
}

export interface SecretRotatedEvent {
  path: string;
  version: number;
}

export interface VaultClientConfig {
  serverUrl: string;
  cacheTtlMs?: number;
  cacheMaxCapacity?: number;
  kafkaBrokers?: string;
}

export interface VaultClient {
  getSecret(path: string): Promise<Secret>;
  getSecretValue(path: string, key: string): Promise<string>;
  listSecrets(pathPrefix: string): Promise<string[]>;
  watchSecret(path: string): AsyncIterable<SecretRotatedEvent>;
}

export class GrpcVaultClient implements VaultClient {
  constructor(config: VaultClientConfig);
  getSecret(path: string): Promise<Secret>;
  getSecretValue(path: string, key: string): Promise<string>;
  listSecrets(pathPrefix: string): Promise<string[]>;
  watchSecret(path: string): AsyncIterable<SecretRotatedEvent>;
  close(): Promise<void>;
}

export class VaultError extends Error {
  constructor(
    message: string,
    public readonly code: 'NOT_FOUND' | 'PERMISSION_DENIED' | 'SERVER_ERROR' | 'TIMEOUT' | 'LEASE_EXPIRED'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/vault_client/`

```
vault_client/
├── pubspec.yaml        # k1s0_vault_client
├── analysis_options.yaml
├── lib/
│   ├── vault_client.dart
│   └── src/
│       ├── client.dart         # VaultClient abstract, GrpcVaultClient
│       ├── secret.dart         # Secret, SecretRotatedEvent
│       ├── config.dart         # VaultClientConfig
│       └── error.dart          # VaultError
└── test/
    └── vault_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  grpc: ^4.0.0
  protobuf: ^3.1.0
```

**使用例**:

```dart
import 'package:k1s0_vault_client/vault_client.dart';

final config = VaultClientConfig(
  serverUrl: 'http://vault-server:8080',
  cacheTtl: Duration(minutes: 10),
);
final client = GrpcVaultClient(config);

// DB パスワードの取得
final dbPassword = await client.getSecretValue(
  'system/database/primary',
  'password',
);

// ローテーション監視
final watcher = await client.watchSecret('system/database/primary');
await for (final event in watcher) {
  print('シークレットローテーション: ${event.path} v${event.version}');
}
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/vault-client/`

```
vault-client/
├── src/
│   ├── VaultClient.csproj
│   ├── IVaultClient.cs             # シークレット操作インターフェース
│   ├── GrpcVaultClient.cs          # gRPC 実装（IMemoryCache・ウォッチャー内蔵）
│   ├── Secret.cs                   # Secret・SecretRotatedEvent
│   ├── VaultClientConfig.cs        # サーバー URL・キャッシュ TTL・Kafka 設定
│   └── VaultException.cs           # 公開例外型
├── tests/
│   ├── VaultClient.Tests.csproj
│   ├── Unit/
│   │   ├── SecretTests.cs
│   │   └── VaultClientCacheTests.cs
│   └── Integration/
│       └── GrpcVaultClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Grpc.Net.Client 2.67 | gRPC クライアント |
| Microsoft.Extensions.Caching.Memory 9.0 | TTL 付きインメモリキャッシュ |
| Confluent.Kafka 2.6 | Kafka コンシューマー（ウォッチャー）|
| Google.Protobuf 3.29 | Protobuf シリアライゼーション |

**名前空間**: `K1s0.System.VaultClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IVaultClient` | interface | シークレット操作の抽象インターフェース |
| `GrpcVaultClient` | class | gRPC 経由の vault-server 接続実装（キャッシュ・ウォッチャー内蔵）|
| `Secret` | record | パス・データマップ・バージョン・作成日時 |
| `SecretRotatedEvent` | record | ローテーション通知（パス・新バージョン）|
| `VaultClientConfig` | record | サーバー URL・キャッシュ TTL・Kafka ブローカー設定 |
| `VaultException` | class | シークレットエラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.VaultClient;

public record Secret(
    string Path,
    IReadOnlyDictionary<string, string> Data,
    long Version,
    DateTimeOffset CreatedAt);

public record SecretRotatedEvent(string Path, long Version);

public interface IVaultClient : IAsyncDisposable
{
    Task<Secret> GetSecretAsync(string path, CancellationToken ct = default);
    Task<string> GetSecretValueAsync(string path, string key, CancellationToken ct = default);
    Task<IReadOnlyList<string>> ListSecretsAsync(string pathPrefix, CancellationToken ct = default);
    IAsyncEnumerable<SecretRotatedEvent> WatchSecretAsync(string path, CancellationToken ct = default);
}

public record VaultClientConfig(
    string ServerUrl,
    TimeSpan? CacheTtl = null,
    int CacheMaxCapacity = 500,
    string? KafkaBrokers = null);

public sealed class GrpcVaultClient : IVaultClient
{
    public GrpcVaultClient(VaultClientConfig config);
    public Task<Secret> GetSecretAsync(string path, CancellationToken ct = default);
    public Task<string> GetSecretValueAsync(string path, string key, CancellationToken ct = default);
    public Task<IReadOnlyList<string>> ListSecretsAsync(string pathPrefix, CancellationToken ct = default);
    public IAsyncEnumerable<SecretRotatedEvent> WatchSecretAsync(string path, CancellationToken ct = default);
    public ValueTask DisposeAsync();
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0VaultClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開 API

```swift
public struct Secret: Sendable {
    public let path: String
    public let data: [String: String]
    public let version: Int64
    public let createdAt: Date
}

public struct SecretRotatedEvent: Sendable {
    public let path: String
    public let version: Int64
}

public protocol VaultClient: Sendable {
    func getSecret(path: String) async throws -> Secret
    func getSecretValue(path: String, key: String) async throws -> String
    func listSecrets(pathPrefix: String) async throws -> [String]
    func watchSecret(path: String) -> AsyncThrowingStream<SecretRotatedEvent, Error>
}

public struct VaultClientConfig: Sendable {
    public let serverUrl: String
    public let cacheTtl: Duration
    public let cacheMaxCapacity: Int
    public init(serverUrl: String, cacheTtl: Duration = .seconds(600), cacheMaxCapacity: Int = 500)
}
```

### エラー型

```swift
public enum VaultError: Error, Sendable {
    case notFound(path: String)
    case permissionDenied(path: String)
    case serverError(underlying: Error)
    case timeout
    case leaseExpired(path: String)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/vault_client/`

### パッケージ構造

```
vault_client/
├── pyproject.toml
├── src/
│   └── k1s0_vault_client/
│       ├── __init__.py           # 公開 API（再エクスポート）
│       ├── client.py             # VaultClient ABC・GrpcVaultClient（TTL キャッシュ・ウォッチャー内蔵）
│       ├── secret.py             # Secret dataclass・SecretRotatedEvent
│       ├── config.py             # VaultClientConfig
│       ├── exceptions.py         # VaultError
│       └── py.typed
└── tests/
    ├── test_vault_client.py
    └── test_secret_watcher.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `VaultClient` | ABC | シークレット操作抽象基底クラス |
| `GrpcVaultClient` | class | gRPC 経由の vault-server 接続実装（TTL キャッシュ・ウォッチャー内蔵）|
| `Secret` | dataclass | パス・データマップ・バージョン・作成日時 |
| `SecretRotatedEvent` | dataclass | ローテーション通知（パス・新バージョン）|
| `VaultClientConfig` | dataclass | サーバー URL・キャッシュ TTL・Kafka ブローカー |
| `VaultError` | Exception | シークレットエラー基底クラス |

### 使用例

```python
import asyncio
from k1s0_vault_client import GrpcVaultClient, VaultClientConfig

config = VaultClientConfig(
    server_url="http://vault-server:8080",
    cache_ttl=600.0,  # 10分
    cache_max_capacity=500,
)
client = GrpcVaultClient(config)

# DB パスワードの取得
db_password = await client.get_secret_value(
    "system/database/primary", "password"
)

# シークレット一覧
paths = await client.list_secrets("system/")
for path in paths:
    print(f"シークレットパス: {path}")

# ローテーション監視
async for event in client.watch_secret("system/database/primary"):
    print(f"ローテーション: {event.path} v{event.version}")
    # コネクションプールを再初期化するなどの処理
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| grpcio | >=1.70 | gRPC クライアント |
| grpcio-tools | >=1.70 | Protobuf コード生成 |
| cachetools | >=5.5 | TTL 付きインメモリキャッシュ |
| kafka-python | >=2.0 | Kafka コンシューマー（ウォッチャー）|

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_secret_data_access() {
        let mut data = HashMap::new();
        data.insert("password".to_string(), "s3cr3t".to_string());

        let secret = Secret {
            path: "system/database/primary".to_string(),
            data,
            version: 1,
            created_at: Utc::now(),
        };

        assert_eq!(secret.data.get("password").unwrap(), "s3cr3t");
    }

    #[test]
    fn test_vault_error_not_found() {
        let err = VaultError::NotFound("system/missing".to_string());
        assert!(matches!(err, VaultError::NotFound(_)));
    }

    #[test]
    fn test_vault_error_permission_denied() {
        let err = VaultError::PermissionDenied("system/secret".to_string());
        assert!(matches!(err, VaultError::PermissionDenied(_)));
    }
}
```

### 統合テスト

- `testcontainers` で vault-server コンテナを起動して実際の get/list フローを検証
- TTL 経過後にキャッシュが無効化されてサーバーへの再取得が発生することを確認
- 存在しないパスで `NotFound` エラーが返ることを確認
- `watch_secret` がローテーションイベントを正しく受信することを確認（Kafka 連携）

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestVaultClient {}
    #[async_trait]
    impl VaultClient for TestVaultClient {
        async fn get_secret(&self, path: &str) -> Result<Secret, VaultError>;
        async fn get_secret_value(&self, path: &str, key: &str) -> Result<String, VaultError>;
        async fn list_secrets(&self, path_prefix: &str) -> Result<Vec<String>, VaultError>;
        async fn watch_secret(&self, path: &str) -> Result<SecretWatcher, VaultError>;
    }
}

#[tokio::test]
async fn test_db_service_fetches_credentials_from_vault() {
    let mut mock = MockTestVaultClient::new();
    mock.expect_get_secret_value()
        .withf(|p, k| p == "system/database/primary" && k == "password")
        .once()
        .returning(|_, _| Ok("db-password-123".to_string()));

    let db_service = DatabaseService::new(Arc::new(mock));
    let conn = db_service.connect().await.unwrap();
    assert!(conn.is_valid());
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-vault-server設計](system-vault-server設計.md) — シークレット管理サーバー設計
- [system-library-encryption設計](system-library-encryption設計.md) — 暗号化ライブラリ（シークレットの暗号化保存）
- [system-library-cache設計](system-library-cache設計.md) — k1s0-cache ライブラリ（キャッシュ基盤）
- [system-library-kafka設計](system-library-kafka設計.md) — Kafka コンシューマー（`k1s0.system.vault.secret_rotated.v1`）
