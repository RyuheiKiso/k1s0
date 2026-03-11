# k1s0-statestore ライブラリ設計

## 概要

StateStore Building Block ライブラリ。キーバリュー型の状態管理を統一インターフェースで抽象化する。ETag による楽観ロック（CAS: Compare-And-Swap）を標準サポートし、並行アクセス時のデータ整合性を保証する。RedisStateStore は内部で k1s0-cache をラップして利用する。

**配置先**: `regions/system/library/rust/statestore/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `StateStore` | トレイト | StateStore Building Block インターフェース（Component 継承） |
| `StateEntry` | 構造体 | 状態エントリ（key・value・etag） |
| `ETag` | 構造体 | 楽観ロック用バージョンタグ |
| `StateStoreError` | enum | `KeyNotFound`・`ETagMismatch`・`ConnectionFailed`・`SerializeFailed` |
| `RedisStateStore` | 構造体 | Redis 実装（k1s0-cache ラッパー） |
| `PostgresStateStore` | 構造体 | PostgreSQL 実装 |
| `InMemoryStateStore` | 構造体 | InMemory 実装（テスト用） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-statestore"
version = "0.1.0"
edition = "2021"

[features]
redis = ["k1s0-cache"]
postgres = ["sqlx"]
mock = []

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
k1s0-cache = { path = "../cache", optional = true }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-statestore = { path = "../../system/library/rust/statestore" }`

**モジュール構成**:

```
statestore/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── traits.rs       # StateStore トレイト定義
│   ├── entry.rs        # StateEntry・ETag
│   ├── error.rs        # StateStoreError
│   ├── redis.rs        # RedisStateStore（feature = "redis"）
│   ├── postgres.rs     # PostgresStateStore（feature = "postgres"）
│   └── in_memory.rs    # InMemoryStateStore
└── Cargo.toml
```

**StateStore トレイト**:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait StateStore: Component + Send + Sync {
    /// キーに対応する状態を取得する。
    async fn get(&self, key: &str) -> Result<Option<StateEntry>, StateStoreError>;

    /// キーに状態を設定する。etag が指定された場合は CAS 操作となる。
    async fn set(
        &self,
        key: &str,
        value: serde_json::Value,
        etag: Option<&ETag>,
    ) -> Result<ETag, StateStoreError>;

    /// キーの状態を削除する。etag が指定された場合は CAS 操作となる。
    async fn delete(&self, key: &str, etag: Option<&ETag>) -> Result<(), StateStoreError>;

    /// 複数キーの状態を一括取得する。
    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<StateEntry>, StateStoreError>;

    /// 複数キーの状態を一括設定する。
    async fn bulk_set(
        &self,
        entries: &[(&str, serde_json::Value, Option<&ETag>)],
    ) -> Result<Vec<ETag>, StateStoreError>;
}
```

**ETag 楽観ロック**:

```rust
/// ETag は状態のバージョンを表す。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ETag(pub String);

/// StateEntry はキーと値、ETag を保持する。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub etag: ETag,
}
```

CAS セマンティクス:

1. `get` でエントリと ETag を取得
2. `set` / `delete` 時に取得した ETag を渡す
3. 他のプロセスが先に更新していた場合、ETag が不一致となり `StateStoreError::ETagMismatch` を返す
4. ETag を `None` で渡した場合は無条件書き込み（Last-Writer-Wins）

**使用例**:

```rust
use k1s0_statestore::{StateStore, InMemoryStateStore, StateStoreError};

let store = InMemoryStateStore::new();

// 状態の設定
let etag = store.set("user:123", serde_json::json!({"name": "Alice"}), None).await?;

// 状態の取得
let entry = store.get("user:123").await?.unwrap();
assert_eq!(entry.value["name"], "Alice");

// ETag 付き更新（楽観ロック）
let new_etag = store.set(
    "user:123",
    serde_json::json!({"name": "Alice", "age": 30}),
    Some(&entry.etag),
).await?;

// 古い ETag での更新は失敗
match store.set("user:123", serde_json::json!({}), Some(&etag)).await {
    Err(StateStoreError::ETagMismatch { .. }) => println!("concurrent update detected"),
    _ => panic!("expected ETagMismatch"),
}
```

## Go 実装

**配置先**: `regions/system/library/go/statestore/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```go
package statestore

import "context"

// StateStore は状態管理抽象化インターフェース。
type StateStore interface {
    buildingblocks.Component
    Get(ctx context.Context, key string) (*StateEntry, error)
    Set(ctx context.Context, key string, value []byte, etag *ETag) (*ETag, error)
    Delete(ctx context.Context, key string, etag *ETag) error
    BulkGet(ctx context.Context, keys []string) ([]*StateEntry, error)
    BulkSet(ctx context.Context, entries []*SetRequest) ([]*ETag, error)
}

type StateEntry struct {
    Key   string `json:"key"`
    Value []byte `json:"value"`
    ETag  ETag   `json:"etag"`
}

type ETag struct {
    Value string `json:"value"`
}

type SetRequest struct {
    Key   string
    Value []byte
    ETag  *ETag
}

type ErrorKind int

const (
    KeyNotFound ErrorKind = iota
    ETagMismatch
    ConnectionFailed
    SerializeFailed
)

type StateStoreError struct {
    Kind    ErrorKind
    Message string
    Err     error
}

func (e *StateStoreError) Error() string
func (e *StateStoreError) Unwrap() error

// --- 実装 ---

type RedisStateStore struct { /* k1s0-cache ラッパー */ }
func NewRedisStateStore() *RedisStateStore

type PostgresStateStore struct { /* PostgreSQL */ }
func NewPostgresStateStore() *PostgresStateStore

type InMemoryStateStore struct { /* テスト用 */ }
func NewInMemoryStateStore() *InMemoryStateStore
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/statestore/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
import { Component } from '@k1s0/building-blocks';

export interface StateEntry {
  readonly key: string;
  readonly value: unknown;
  readonly etag: string;
}

export interface ETag {
  readonly value: string;
}

export type StateStoreErrorCode =
  | 'KEY_NOT_FOUND'
  | 'ETAG_MISMATCH'
  | 'CONNECTION_FAILED'
  | 'SERIALIZE_FAILED';

export class StateStoreError extends Error {
  constructor(
    message: string,
    public readonly code: StateStoreErrorCode,
  ) {
    super(message);
  }
}

export interface StateStore extends Component {
  get(key: string): Promise<StateEntry | null>;
  set(key: string, value: unknown, etag?: string): Promise<string>;
  delete(key: string, etag?: string): Promise<void>;
  bulkGet(keys: string[]): Promise<StateEntry[]>;
  bulkSet(
    entries: Array<{ key: string; value: unknown; etag?: string }>,
  ): Promise<string[]>;
}

export class RedisStateStore implements StateStore { /* ... */ }
export class PostgresStateStore implements StateStore { /* ... */ }
export class InMemoryStateStore implements StateStore { /* ... */ }
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/statestore/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
import 'package:k1s0_building_blocks/component.dart';

class StateEntry {
  final String key;
  final dynamic value;
  final String etag;
  const StateEntry({required this.key, required this.value, required this.etag});
}

enum StateStoreErrorCode {
  keyNotFound,
  etagMismatch,
  connectionFailed,
  serializeFailed,
}

class StateStoreError implements Exception {
  final String message;
  final StateStoreErrorCode code;
  const StateStoreError(this.message, this.code);
}

abstract class StateStore implements Component {
  Future<StateEntry?> get(String key);
  Future<String> set(String key, dynamic value, {String? etag});
  Future<void> delete(String key, {String? etag});
  Future<List<StateEntry>> bulkGet(List<String> keys);
  Future<List<String>> bulkSet(List<Map<String, dynamic>> entries);
}

class RedisStateStore implements StateStore { /* k1s0-cache ラッパー */ }
class PostgresStateStore implements StateStore { /* PostgreSQL */ }
class InMemoryStateStore implements StateStore { /* テスト用 */ }
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト

InMemoryStateStore を活用し、StateStore トレイトの全操作を検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_set_delete() {
        let store = InMemoryStateStore::new();

        // 存在しないキーの取得
        assert!(store.get("key1").await.unwrap().is_none());

        // 設定と取得
        let etag = store.set("key1", serde_json::json!("value1"), None).await.unwrap();
        let entry = store.get("key1").await.unwrap().unwrap();
        assert_eq!(entry.value, serde_json::json!("value1"));

        // 削除
        store.delete("key1", Some(&etag)).await.unwrap();
        assert!(store.get("key1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_etag_optimistic_lock() {
        let store = InMemoryStateStore::new();
        let etag1 = store.set("key1", serde_json::json!("v1"), None).await.unwrap();
        let _etag2 = store.set("key1", serde_json::json!("v2"), Some(&etag1)).await.unwrap();

        // 古い ETag での更新は ETagMismatch
        let result = store.set("key1", serde_json::json!("v3"), Some(&etag1)).await;
        assert!(matches!(result, Err(StateStoreError::ETagMismatch { .. })));
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let store = InMemoryStateStore::new();
        let etags = store.bulk_set(&[
            ("a", serde_json::json!(1), None),
            ("b", serde_json::json!(2), None),
        ]).await.unwrap();
        assert_eq!(etags.len(), 2);

        let entries = store.bulk_get(&["a", "b", "c"]).await.unwrap();
        assert_eq!(entries.len(), 2); // "c" は存在しない
    }
}
```

### 統合テスト

- testcontainers で Redis・PostgreSQL を起動し、実装の動作を検証
- 並行 CAS 操作での ETagMismatch 発生を確認
- 大量データでの bulk_get / bulk_set パフォーマンスを計測

### コントラクトテスト

全 StateStore 実装（Redis・PostgreSQL・InMemory）が同一の振る舞い仕様を満たすことを共通テストスイートで検証する。

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [Building Blocks 概要](../_common/building-blocks.md) — BB 設計思想・共通インターフェース
- [system-library-cache設計](cache.md) — Redis キャッシュライブラリ（RedisStateStore が内部利用）
- [system-library-distributed-lock設計](distributed-lock.md) — 分散ロック
- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
