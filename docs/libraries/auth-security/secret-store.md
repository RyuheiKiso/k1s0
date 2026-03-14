# k1s0-secretstore ライブラリ設計

> **実装形態**: Go は独立パッケージ、Rust は bb-secretstore、TypeScript/Dart は building-blocks に統合
> - Go: `regions/system/library/go/secret-store/` (module: `github.com/k1s0-platform/system-library-go-secret-store`)
> - Rust: `regions/system/library/rust/bb-secretstore/`
> - TypeScript/Dart: `regions/system/library/{typescript,dart}/building-blocks/`

## 概要

SecretStore Building Block ライブラリ。シークレット取得を統一インターフェースで抽象化する。VaultSecretStore は内部で k1s0-vault-client をラップして利用する。環境変数やファイルからのシークレット取得も同一インターフェースで扱える。

**配置先**: `regions/system/library/rust/bb-secretstore/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SecretStore` | トレイト | SecretStore Building Block インターフェース（Component 継承） |
| `Secret` | 構造体 | シークレットデータ（key・value・metadata） |
| `SecretStoreError` | enum | `SecretNotFound`・`AccessDenied`・`ConnectionFailed`・`ParseFailed` |
| `VaultSecretStore` | 構造体 | Vault 実装（k1s0-vault-client ラッパー） |
| `InMemorySecretStore` | 構造体 | InMemory 実装（テスト用） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "bb-secretstore"
version = "0.1.0"
edition = "2021"

[features]
default = []
vault = ["k1s0-vault-client"]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
k1s0-bb-core = { path = "../bb-core" }
k1s0-vault-client = { path = "../vault-client", optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
tracing = "0.1"
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `bb-secretstore = { path = "../../system/library/rust/bb-secretstore" }`

**モジュール構成**:

```
bb-secretstore/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── traits.rs       # SecretStore トレイト定義・Secret 構造体
│   ├── error.rs        # SecretStoreError
│   ├── vault.rs        # VaultSecretStore（feature = "vault"）
│   └── memory.rs       # InMemorySecretStore
└── Cargo.toml
```

**SecretStore トレイト**:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SecretStore: Component + Send + Sync {
    /// キーに対応するシークレットを取得する。
    async fn get(&self, key: &str) -> Result<Secret, SecretStoreError>;

    /// 複数キーのシークレットを一括取得する。
    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<Secret>, SecretStoreError>;

    /// リソースを解放する。
    async fn close(&self) -> Result<(), SecretStoreError>;
}
```

**Secret 構造体**:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
    pub metadata: std::collections::HashMap<String, String>,
}
```

**VaultSecretStore 内部構造**:

```rust
pub struct VaultSecretStore {
    client: k1s0_vault_client::VaultClient,
}

#[async_trait]
impl Component for VaultSecretStore {
    fn name(&self) -> &str { "vault" }
    fn version(&self) -> &str { "1.0.0" }

    async fn init(&mut self, metadata: Metadata) -> Result<(), ComponentError> {
        let address = metadata.properties.get("address")
            .ok_or_else(|| ComponentError::InitFailed("address required".into()))?;
        // k1s0-vault-client で接続初期化
        self.client = k1s0_vault_client::VaultClient::new(address)?;
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        Ok(())
    }
}
```

**使用例**:

```rust
use k1s0_secretstore::{SecretStore, InMemorySecretStore, Secret};

// テスト用 InMemory
let mut store = InMemorySecretStore::new();
store.add_secret("db-password", "super-secret-123");
store.add_secret("api-key", "ak_test_abcdef");

let secret = store.get("db-password").await?;
assert_eq!(secret.value, "super-secret-123");

let secrets = store.bulk_get(&["db-password", "api-key"]).await?;
assert_eq!(secrets.len(), 2);
```

## Go 実装

**配置先**: `regions/system/library/go/building-blocks/`

**主要インターフェース**:

```go
package secretstore

import "context"

// SecretStore はシークレット取得の抽象化インターフェース。
type SecretStore interface {
    buildingblocks.Component
    Get(ctx context.Context, key string) (*Secret, error)
    BulkGet(ctx context.Context, keys []string) ([]*Secret, error)
    Close(ctx context.Context) error
}

type Secret struct {
    Key      string            `json:"key"`
    Value    string            `json:"value"`
    Metadata map[string]string `json:"metadata"`
}

type ErrorKind int

const (
    SecretNotFound ErrorKind = iota
    AccessDenied
    ConnectionFailed
    ParseFailed
)

type SecretStoreError struct {
    Kind    ErrorKind
    Message string
    Err     error
}

func (e *SecretStoreError) Error() string
func (e *SecretStoreError) Unwrap() error

// --- 実装 ---

// VaultSecretStore: VaultClientIface インターフェース経由で注入（k1s0-vault-client 互換）。
type VaultSecretStore struct{}
func NewVaultSecretStore(name string, client VaultClientIface) *VaultSecretStore

// EnvSecretStore: prefix+key の環境変数を参照（外部依存なし）。
type EnvSecretStore struct{}
func NewEnvSecretStore(prefix string) *EnvSecretStore

// FileSecretStore: ディレクトリ内ファイルを読み取る（Kubernetes Secrets マウント互換）。
type FileSecretStore struct{}
func NewFileSecretStore(dir string) *FileSecretStore

// InMemorySecretStore: テスト・開発用（Put() ヘルパー付き）。
type InMemorySecretStore struct{}
func NewInMemorySecretStore() *InMemorySecretStore
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/building-blocks/`

> **現在の実装**: `InMemorySecretStore` のみ。本番バックエンド（Vault・Env・File）は Go/Rust 側で提供する。

**主要 API**:

```typescript
import { Component } from '@k1s0/building-blocks';

export interface Secret {
  readonly key: string;
  readonly value: string;
  readonly metadata: Record<string, string>;
}

export type SecretStoreErrorCode =
  | 'SECRET_NOT_FOUND'
  | 'ACCESS_DENIED'
  | 'CONNECTION_FAILED'
  | 'PARSE_FAILED';

export class SecretStoreError extends Error {
  constructor(
    message: string,
    public readonly code: SecretStoreErrorCode,
  ) {
    super(message);
  }
}

export interface SecretStore extends Component {
  get(key: string): Promise<Secret>;
  bulkGet(keys: string[]): Promise<Secret[]>;
  close(): Promise<void>;
}

export class InMemorySecretStore implements SecretStore { /* テスト・開発用（put() ヘルパー付き） */ }
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/building-blocks/`

> **現在の実装**: `InMemorySecretStore` のみ。本番バックエンドは Go/Rust 側で提供する。

**主要 API**:

```dart
import 'package:k1s0_building_blocks/component.dart';

class Secret {
  final String key;
  final String value;
  final Map<String, String> metadata;
  const Secret({required this.key, required this.value, this.metadata = const {}});
}

enum SecretStoreErrorCode {
  secretNotFound,
  accessDenied,
  connectionFailed,
  parseFailed,
}

class SecretStoreError implements Exception {
  final String message;
  final SecretStoreErrorCode code;
  const SecretStoreError(this.message, this.code);
}

abstract class SecretStore implements Component {
  Future<Secret> get(String key);
  Future<List<Secret>> bulkGet(List<String> keys);
  Future<void> close();
}

class InMemorySecretStore implements SecretStore { /* テスト・開発用（put() ヘルパー付き） */ }
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト

InMemorySecretStore を活用し、SecretStore トレイトの振る舞いを検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_existing_secret() {
        let mut store = InMemorySecretStore::new();
        store.add_secret("db-password", "secret123");

        let secret = store.get("db-password").await.unwrap();
        assert_eq!(secret.key, "db-password");
        assert_eq!(secret.value, "secret123");
    }

    #[tokio::test]
    async fn test_get_nonexistent_secret() {
        let store = InMemorySecretStore::new();
        let result = store.get("nonexistent").await;
        assert!(matches!(result, Err(SecretStoreError::SecretNotFound { .. })));
    }

    #[tokio::test]
    async fn test_bulk_get() {
        let mut store = InMemorySecretStore::new();
        store.add_secret("key1", "val1");
        store.add_secret("key2", "val2");

        let secrets = store.bulk_get(&["key1", "key2"]).await.unwrap();
        assert_eq!(secrets.len(), 2);
    }
}
```

### 統合テスト

- testcontainers で Vault を起動し、VaultSecretStore の動作を検証
- シークレットの作成・取得・更新検知を確認
- アクセス権限不足時の AccessDenied エラーを検証

### コントラクトテスト

全 SecretStore 実装（Vault・Env・File・InMemory）が同一の振る舞い仕様を満たすことを共通テストスイートで検証する。

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [Building Blocks 概要](../_common/building-blocks.md) — BB 設計思想・共通インターフェース
- [system-library-vault-client設計](vault-client.md) — Vault クライアントライブラリ（VaultSecretStore が内部利用）
- [system-library-encryption設計](encryption.md) — 暗号化・復号化ユーティリティ
- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
